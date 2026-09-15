//! Layered global preferences and repository message-policy configuration.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::commit::{Capitalization, IssueStyle, MessagePolicy, TerminalPunctuation};

#[doc(inline)]
pub use crate::settings::{AccentColor, UiSections};

const SCHEMA_VERSION: u32 = 1;

/// The complete configuration after defaults, global, and repository layers merge.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub ui: UiPreferences,
    pub message: MessagePolicy,
    pub defaults: ComposerDefaultCommands,
}

/// User-interface preferences, which are intentionally global only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiPreferences {
    pub sign: bool,
    pub sections: UiSections,
    pub accent_color: AccentColor,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            sign: true,
            sections: UiSections::default(),
            accent_color: AccentColor::default(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct GlobalFile {
    schema_version: Option<u32>,
    ui: Option<UiPreferencesPatch>,
    message: Option<MessagePolicyPatch>,
    defaults: Option<ComposerDefaultCommands>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct RepositoryFile {
    schema_version: Option<u32>,
    message: Option<MessagePolicyPatch>,
}

/// Global-only commands used to prefill editable composer fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ComposerDefaultCommands {
    #[serde(rename = "type")]
    pub commit_type: Option<DefaultCommand>,
    pub scope: Option<DefaultCommand>,
    pub body: Option<DefaultCommand>,
    pub issue: Option<DefaultCommand>,
}

/// An executable followed by literal arguments. No shell parsing is performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultCommand {
    pub command: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct UiPreferencesPatch {
    sign: Option<bool>,
    sections: Option<UiSectionsPatch>,
    accent_color: Option<AccentColor>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct UiSectionsPatch {
    #[serde(rename = "type")]
    commit_type: Option<bool>,
    scope: Option<bool>,
    breaking: Option<bool>,
    staged_changes: Option<bool>,
    body: Option<bool>,
    footers: Option<bool>,
    issue: Option<bool>,
    sign: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct MessagePolicyPatch {
    types: Option<Vec<String>>,
    scope_suggestions: Option<Vec<String>>,
    subject: Option<SubjectPolicyPatch>,
    issue: Option<IssuePolicyPatch>,
    format: Option<MessageFormatPatch>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct SubjectPolicyPatch {
    max_length: Option<usize>,
    capitalization: Option<Capitalization>,
    terminal_punctuation: Option<TerminalPunctuation>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct IssuePolicyPatch {
    prefix: Option<String>,
    style: Option<IssueStyle>,
}

/// Returns the optional global configuration file location.
pub fn global_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|directory| directory.join("cocommit").join("config.toml"))
}

/// Returns the repository policy file at the Git work-tree root.
pub fn repository_config_path(work_tree_root: &Path) -> PathBuf {
    work_tree_root.join(".cocommit.toml")
}

/// Loads defaults, global configuration, and repository policy in precedence order.
pub fn load(work_tree_root: &Path) -> Result<Config> {
    load_from_paths(global_config_path().as_deref(), work_tree_root)
}

fn load_from_paths(global_path: Option<&Path>, work_tree_root: &Path) -> Result<Config> {
    let mut config = Config::default();
    if let Some(path) = global_path {
        merge_global(&mut config, load_global_from_path(path)?);
    }
    merge_repository(
        &mut config,
        load_repository_from_path(&repository_config_path(work_tree_root))?,
    );
    validate(&config)?;
    Ok(config)
}

fn load_global_from_path(path: &Path) -> Result<GlobalFile> {
    let contents = read_optional(path, "global configuration")?;
    contents.map_or_else(
        || Ok(GlobalFile::default()),
        |contents| parse_global(&contents, path),
    )
}

fn load_repository_from_path(path: &Path) -> Result<RepositoryFile> {
    let contents = read_optional(path, "repository configuration")?;
    contents.map_or_else(
        || Ok(RepositoryFile::default()),
        |contents| parse_repository(&contents, path),
    )
}

fn read_optional(path: &Path, layer: &str) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("failed to read {layer} file at {}", path.display()))
        }
    }
}

fn parse_global(contents: &str, path: &Path) -> Result<GlobalFile> {
    let file: GlobalFile = toml::from_str(contents).with_context(|| {
        format!(
            "failed to parse global configuration file at {}",
            path.display()
        )
    })?;
    match file.schema_version {
        None if file.ui.is_none() && file.message.is_none() && file.defaults.is_none() => Ok(file),
        None => bail!(
            "global configuration file at {} uses schema fields but has no schema_version",
            path.display()
        ),
        Some(SCHEMA_VERSION) => {
            validate_message_patch(file.message.as_ref()).with_context(|| {
                format!("invalid global configuration file at {}", path.display())
            })?;
            validate_default_commands(file.defaults.as_ref()).with_context(|| {
                format!("invalid global configuration file at {}", path.display())
            })?;
            Ok(file)
        }
        Some(version) => bail!(
            "global configuration file at {} uses unsupported schema_version {version}",
            path.display()
        ),
    }
}

fn parse_repository(contents: &str, path: &Path) -> Result<RepositoryFile> {
    let file: RepositoryFile = toml::from_str(contents).with_context(|| {
        format!(
            "failed to parse repository configuration file at {}",
            path.display()
        )
    })?;
    match file.schema_version {
        None if file.message.is_none() => Ok(file),
        Some(SCHEMA_VERSION) => {
            validate_message_patch(file.message.as_ref()).with_context(|| {
                format!(
                    "invalid repository configuration file at {}",
                    path.display()
                )
            })?;
            Ok(file)
        }
        None => bail!(
            "repository configuration file at {} has message policy but no schema_version",
            path.display()
        ),
        Some(version) => bail!(
            "repository configuration file at {} uses unsupported schema_version {version}",
            path.display()
        ),
    }
}

fn validate_message_patch(patch: Option<&MessagePolicyPatch>) -> Result<()> {
    let Some(patch) = patch else {
        return Ok(());
    };
    if patch.types.as_ref().is_some_and(Vec::is_empty) {
        bail!("message policy types cannot be empty");
    }
    if patch.types.as_ref().is_some_and(|types| {
        types.iter().any(|commit_type| {
            commit_type.trim() != commit_type
                || commit_type.is_empty()
                || commit_type.contains(['\n', '\r'])
                || commit_type.chars().any(char::is_whitespace)
                || commit_type
                    .chars()
                    .any(|character| matches!(character, '(' | ')' | '!' | ':'))
        })
    }) {
        bail!("message policy types contain an invalid value");
    }
    if patch.scope_suggestions.as_ref().is_some_and(|scopes| {
        scopes.iter().any(|scope| {
            scope.is_empty() || scope.trim() != scope || scope.contains(['\n', '\r', '(', ')'])
        })
    }) {
        bail!("message policy scope_suggestions contain an invalid value");
    }
    if patch
        .issue
        .as_ref()
        .and_then(|issue| issue.prefix.as_ref())
        .is_some_and(|prefix| prefix.chars().any(char::is_control))
    {
        bail!("message policy issue prefix cannot contain control characters");
    }
    if patch
        .subject
        .as_ref()
        .and_then(|subject| subject.max_length)
        == Some(0)
    {
        bail!("message policy subject max_length must be greater than zero");
    }
    if patch
        .format
        .as_ref()
        .and_then(|format| format.separator.as_ref())
        .is_some_and(|separator| {
            separator.chars().count() > 8
                || separator
                    .chars()
                    .any(|character| character.is_control() || character.is_whitespace())
        })
    {
        bail!("message format separator must be at most 8 non-whitespace characters");
    }
    Ok(())
}

fn validate_default_commands(commands: Option<&ComposerDefaultCommands>) -> Result<()> {
    let Some(commands) = commands else {
        return Ok(());
    };
    for (field, command) in [
        ("type", commands.commit_type.as_ref()),
        ("scope", commands.scope.as_ref()),
        ("body", commands.body.as_ref()),
        ("issue", commands.issue.as_ref()),
    ] {
        let Some(command) = command else {
            continue;
        };
        if command.command.is_empty() || command.command[0].trim().is_empty() {
            bail!("defaults.{field}.command must contain an executable");
        }
        if command.command.len() > 64
            || command
                .command
                .iter()
                .any(|argument| argument.len() > 4096 || argument.contains('\0'))
        {
            bail!("defaults.{field}.command contains too many or invalid arguments");
        }
    }
    Ok(())
}

fn merge_global(config: &mut Config, file: GlobalFile) {
    if let Some(ui) = file.ui {
        merge_ui(&mut config.ui, ui);
    }
    if let Some(message) = file.message {
        merge_message(&mut config.message, message);
    }
    if let Some(defaults) = file.defaults {
        if defaults.commit_type.is_some() {
            config.defaults.commit_type = defaults.commit_type;
        }
        if defaults.scope.is_some() {
            config.defaults.scope = defaults.scope;
        }
        if defaults.body.is_some() {
            config.defaults.body = defaults.body;
        }
        if defaults.issue.is_some() {
            config.defaults.issue = defaults.issue;
        }
    }
}

fn merge_repository(config: &mut Config, file: RepositoryFile) {
    if let Some(message) = file.message {
        merge_message(&mut config.message, message);
    }
}

fn merge_ui(ui: &mut UiPreferences, patch: UiPreferencesPatch) {
    if let Some(sign) = patch.sign {
        ui.sign = sign;
    }
    if let Some(accent_color) = patch.accent_color {
        ui.accent_color = accent_color;
    }
    if let Some(sections) = patch.sections {
        if let Some(commit_type) = sections.commit_type {
            ui.sections.commit_type = commit_type;
        }
        if let Some(scope) = sections.scope {
            ui.sections.scope = scope;
        }
        if let Some(breaking) = sections.breaking {
            ui.sections.breaking = breaking;
        }
        if let Some(staged_changes) = sections.staged_changes {
            ui.sections.staged_changes = staged_changes;
        }
        if let Some(body) = sections.body {
            ui.sections.body = body;
        }
        if let Some(footers) = sections.footers {
            ui.sections.footers = footers;
        }
        if let Some(issue) = sections.issue {
            ui.sections.issue = issue;
        }
        if let Some(sign) = sections.sign {
            ui.sections.sign = sign;
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct MessageFormatPatch {
    separator: Option<String>,
}

fn merge_message(message: &mut MessagePolicy, patch: MessagePolicyPatch) {
    if let Some(types) = patch.types {
        message.types = types;
        message.types_are_restricted = true;
    }
    if let Some(scope_suggestions) = patch.scope_suggestions {
        message.scope_suggestions = scope_suggestions;
    }
    if let Some(subject) = patch.subject {
        if let Some(max_length) = subject.max_length {
            message.subject.max_length = Some(max_length);
        }
        if let Some(capitalization) = subject.capitalization {
            message.subject.capitalization = capitalization;
        }
        if let Some(terminal_punctuation) = subject.terminal_punctuation {
            message.subject.terminal_punctuation = terminal_punctuation;
        }
    }
    if let Some(issue) = patch.issue {
        if let Some(prefix) = issue.prefix {
            message.issue.prefix = prefix;
        }
        if let Some(style) = issue.style {
            message.issue.style = style;
        }
    }
    if let Some(format) = patch.format
        && let Some(separator) = format.separator
    {
        message.format.separator = separator;
    }
}

fn validate(config: &Config) -> Result<()> {
    debug_assert!(!config.message.types.is_empty());
    debug_assert!(
        config
            .message
            .types
            .iter()
            .all(|commit_type| !commit_type.trim().is_empty())
    );
    debug_assert_ne!(config.message.subject.max_length, Some(0));
    Ok(())
}

#[cfg(test)]
mod tests;
