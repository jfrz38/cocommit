//! Layered global preferences and repository message-policy configuration.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

/// The complete configuration after defaults, global, and repository layers merge.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub ui: UiPreferences,
    pub message: MessagePolicy,
}

/// User-interface preferences, which are intentionally global only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiPreferences {
    pub sign: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self { sign: true }
    }
}

/// Repository message conventions. Enforcement begins in Iteration 17.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagePolicy {
    pub types: Vec<String>,
    pub scope_suggestions: Vec<String>,
    pub subject: SubjectPolicy,
    pub issue: IssuePolicy,
}

impl Default for MessagePolicy {
    fn default() -> Self {
        Self {
            types: DEFAULT_TYPES.map(str::to_owned).to_vec(),
            scope_suggestions: Vec::new(),
            subject: SubjectPolicy::default(),
            issue: IssuePolicy::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectPolicy {
    pub max_length: Option<usize>,
    pub capitalization: Capitalization,
    pub terminal_punctuation: TerminalPunctuation,
}

impl Default for SubjectPolicy {
    fn default() -> Self {
        Self {
            max_length: None,
            capitalization: Capitalization::Allow,
            terminal_punctuation: TerminalPunctuation::Allow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capitalization {
    Allow,
    Lowercase,
    Uppercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalPunctuation {
    Allow,
    Forbid,
    Require,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuePolicy {
    pub prefix: String,
    pub style: IssueStyle,
}

impl Default for IssuePolicy {
    fn default() -> Self {
        Self {
            prefix: "#".to_owned(),
            style: IssueStyle::Parenthesized,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueStyle {
    Parenthesized,
    Plain,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct GlobalFile {
    schema_version: Option<u32>,
    sign: Option<bool>,
    ui: Option<UiPreferencesPatch>,
    message: Option<MessagePolicyPatch>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct RepositoryFile {
    schema_version: Option<u32>,
    message: Option<MessagePolicyPatch>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct UiPreferencesPatch {
    sign: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct MessagePolicyPatch {
    types: Option<Vec<String>>,
    scope_suggestions: Option<Vec<String>>,
    subject: Option<SubjectPolicyPatch>,
    issue: Option<IssuePolicyPatch>,
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
        None if file.ui.is_none() && file.message.is_none() => Ok(file),
        None => bail!(
            "global configuration file at {} uses schema fields but has no schema_version",
            path.display()
        ),
        Some(0) => bail!(
            "global configuration file at {} uses unsupported schema_version 0; omit schema_version for the legacy sign setting",
            path.display()
        ),
        Some(SCHEMA_VERSION) if file.sign.is_none() => {
            validate_message_patch(file.message.as_ref()).with_context(|| {
                format!("invalid global configuration file at {}", path.display())
            })?;
            Ok(file)
        }
        Some(SCHEMA_VERSION) => bail!(
            "global configuration file at {} cannot combine legacy sign with schema_version {SCHEMA_VERSION}",
            path.display()
        ),
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
        types
            .iter()
            .any(|commit_type| commit_type.trim().is_empty())
    }) {
        bail!("message policy types cannot contain empty values");
    }
    if patch
        .subject
        .as_ref()
        .and_then(|subject| subject.max_length)
        == Some(0)
    {
        bail!("message policy subject max_length must be greater than zero");
    }
    Ok(())
}

fn merge_global(config: &mut Config, file: GlobalFile) {
    if let Some(sign) = file.sign {
        config.ui.sign = sign;
    }
    if let Some(ui) = file.ui {
        merge_ui(&mut config.ui, ui);
    }
    if let Some(message) = file.message {
        merge_message(&mut config.message, message);
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
}

fn merge_message(message: &mut MessagePolicy, patch: MessagePolicyPatch) {
    if let Some(types) = patch.types {
        message.types = types;
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
