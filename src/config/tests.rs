use std::fs;

use super::{Config, config_path, load_from_path, parse};

#[test]
fn defaults_signing_to_enabled() {
    assert_eq!(Config::default(), Config { sign: true });
}

#[test]
fn parses_empty_toml_with_defaults() {
    assert_eq!(parse(""), Ok(Config::default()));
}

#[test]
fn parses_explicit_signing_preference() {
    assert_eq!(parse("sign = true"), Ok(Config { sign: true }));
}

#[test]
fn rejects_malformed_toml() {
    assert!(parse("sign =").is_err());
}

#[test]
fn rejects_unknown_configuration_keys() {
    assert!(parse("unknown = true").is_err());
}

#[test]
fn defaults_when_configuration_file_is_missing() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("config.toml");

    assert_eq!(load_from_path(&path).unwrap(), Config::default());
}

#[test]
fn includes_path_when_configuration_file_cannot_be_read() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("config.toml");
    fs::create_dir(&path).expect("configuration path should be a directory");

    let error =
        load_from_path(&path).expect_err("directory cannot be read as a configuration file");

    assert!(error.to_string().contains(&path.display().to_string()));
}

#[test]
fn global_configuration_path_uses_cocommit_directory() {
    if let Some(path) = config_path() {
        assert!(path.ends_with("cocommit/config.toml"));
    }
}
