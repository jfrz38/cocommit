use std::fs;

use super::{
    Capitalization, Config, IssueStyle, TerminalPunctuation, global_config_path, load_from_paths,
    repository_config_path,
};
use crate::settings::UiSections;

#[test]
fn defaults_preserve_the_existing_signing_and_message_behavior() {
    let config = Config::default();

    assert!(config.ui.sign);
    assert_eq!(config.ui.sections, UiSections::default());
    assert_eq!(config.message.types[0], "feat");
    assert!(!config.message.types_are_restricted);
    assert!(config.message.scope_suggestions.is_empty());
    assert_eq!(config.message.issue.prefix, "#");
    assert_eq!(config.message.issue.style, IssueStyle::Parenthesized);
}

#[test]
fn rejects_empty_scope_suggestions() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let repository = repository_config_path(directory.path());
    fs::write(
        &repository,
        "schema_version = 1\n[message]\nscope_suggestions = [\"\"]",
    )
    .expect("repository configuration should be written");

    let error = load_from_paths(None, directory.path()).unwrap_err();

    assert!(format!("{error:#}").contains("scope_suggestions"));
}

#[test]
fn merges_global_section_preferences_by_field() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let global = directory.path().join("global.toml");
    fs::write(
        &global,
        "schema_version = 1\n[ui.sections]\nbody = false\nissue = false",
    )
    .expect("global configuration should be written");

    let config = load_from_paths(Some(&global), directory.path()).unwrap();

    assert_eq!(
        config.ui.sections,
        UiSections {
            staged_changes: true,
            body: false,
            footers: true,
            issue: false,
        }
    );
}

#[test]
fn accepts_all_optional_sections_hidden_globally() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let global = directory.path().join("global.toml");
    fs::write(
        &global,
        "schema_version = 1\n[ui.sections]\nstaged_changes = false\nbody = false\nfooters = false\nissue = false",
    )
    .expect("global configuration should be written");

    let config = load_from_paths(Some(&global), directory.path()).unwrap();

    assert_eq!(
        config.ui.sections,
        UiSections {
            staged_changes: false,
            body: false,
            footers: false,
            issue: false,
        }
    );
}

#[test]
fn repository_policy_overrides_global_policy_by_field() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let global = directory.path().join("global.toml");
    fs::write(
        &global,
        r#"schema_version = 1

[ui]
sign = false

[message]
types = ["feat", "fix"]
scope_suggestions = ["api", "cli"]

[message.subject]
max_length = 72
capitalization = "lowercase"

[message.issue]
prefix = "PROJ-"
style = "plain"
"#,
    )
    .expect("global configuration should be written");
    fs::write(
        repository_config_path(directory.path()),
        r#"schema_version = 1

[message]
types = ["docs"]
scope_suggestions = []

[message.subject]
terminal_punctuation = "forbid"

[message.issue]
style = "parenthesized"
"#,
    )
    .expect("repository configuration should be written");

    let config = load_from_paths(Some(&global), directory.path()).unwrap();

    assert!(!config.ui.sign);
    assert_eq!(config.message.types, ["docs"]);
    assert!(config.message.types_are_restricted);
    assert!(config.message.scope_suggestions.is_empty());
    assert_eq!(config.message.subject.max_length, Some(72));
    assert_eq!(
        config.message.subject.capitalization,
        Capitalization::Lowercase
    );
    assert_eq!(
        config.message.subject.terminal_punctuation,
        TerminalPunctuation::Forbid
    );
    assert_eq!(config.message.issue.prefix, "PROJ-");
    assert_eq!(config.message.issue.style, IssueStyle::Parenthesized);
}

#[test]
fn missing_configuration_files_use_defaults() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");

    assert_eq!(
        load_from_paths(None, directory.path()).unwrap(),
        Config::default()
    );
}

#[test]
fn reports_configuration_paths_for_read_and_parse_failures() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let global = directory.path().join("global.toml");
    fs::create_dir(&global).expect("configuration path should be a directory");
    let error = load_from_paths(Some(&global), directory.path()).unwrap_err();
    assert!(error.to_string().contains(&global.display().to_string()));

    let repository = repository_config_path(directory.path());
    fs::write(&repository, "schema_version =").expect("repository configuration should be written");
    let error = load_from_paths(None, directory.path()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains(&repository.display().to_string())
    );
}

#[test]
fn rejects_unknown_keys_at_every_schema_level() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let global = directory.path().join("global.toml");
    for contents in [
        "schema_version = 1\nunknown = true",
        "schema_version = 1\n[ui]\nunknown = true",
        "schema_version = 1\n[ui.sections]\nunknown = true",
        "schema_version = 1\n[message]\nunknown = true",
        "schema_version = 1\n[message.subject]\nunknown = true",
        "schema_version = 1\n[message.issue]\nunknown = true",
    ] {
        fs::write(&global, contents).expect("global configuration should be written");
        assert!(load_from_paths(Some(&global), directory.path()).is_err());
    }
}

#[test]
fn rejects_ui_preferences_in_repository_configuration() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let repository = repository_config_path(directory.path());
    fs::write(
        &repository,
        "schema_version = 1\n[ui.sections]\nbody = false",
    )
    .expect("repository configuration should be written");

    assert!(load_from_paths(None, directory.path()).is_err());
}

#[test]
fn rejects_invalid_schema_versions_and_configuration_forms() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let global = directory.path().join("global.toml");
    for contents in [
        "schema_version = 0",
        "schema_version = 2",
        "sign = false",
        "[message]\ntypes = [\"feat\"]",
        "schema_version = 1\nsign = false",
    ] {
        fs::write(&global, contents).expect("global configuration should be written");
        assert!(load_from_paths(Some(&global), directory.path()).is_err());
    }

    fs::write(
        repository_config_path(directory.path()),
        "[message]\ntypes = [\"feat\"]",
    )
    .expect("repository configuration should be written");
    assert!(load_from_paths(None, directory.path()).is_err());
}

#[test]
fn rejects_invalid_policy_values_with_their_repository_path() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let repository = repository_config_path(directory.path());
    fs::write(&repository, "schema_version = 1\n[message]\ntypes = []")
        .expect("repository configuration should be written");

    let error = load_from_paths(None, directory.path()).unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains(&repository.display().to_string()));
    assert!(message.contains("types cannot be empty"));
}

#[test]
fn repository_configuration_path_uses_work_tree_root() {
    let root = std::path::Path::new("repository");
    assert_eq!(repository_config_path(root), root.join(".cocommit.toml"));
}

#[test]
fn global_configuration_path_uses_cocommit_directory() {
    if let Some(path) = global_config_path() {
        assert!(path.ends_with("cocommit/config.toml"));
    }
}
