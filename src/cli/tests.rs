use std::ffi::OsString;

use super::{Command, USAGE, parse};

#[test]
fn accepts_an_empty_argument_list() {
    assert_eq!(parse([]), Ok(Command::Run));
}

#[test]
fn accepts_help_aliases() {
    assert_eq!(parse([OsString::from("-h")]), Ok(Command::Help));
    assert_eq!(parse([OsString::from("--help")]), Ok(Command::Help));
}

#[test]
fn accepts_version_aliases() {
    assert_eq!(parse([OsString::from("-V")]), Ok(Command::Version));
    assert_eq!(parse([OsString::from("--version")]), Ok(Command::Version));
}

#[test]
fn rejects_unknown_and_additional_arguments() {
    assert_eq!(
        parse([OsString::from("--unknown")])
            .expect_err("unknown argument should be rejected")
            .to_string(),
        "unrecognized argument: --unknown"
    );
    assert_eq!(
        parse([OsString::from("--help"), OsString::from("extra")])
            .expect_err("additional argument should be rejected")
            .to_string(),
        "unrecognized argument: --help"
    );
}

#[test]
fn documents_every_supported_option() {
    assert!(USAGE.contains("--help"));
    assert!(USAGE.contains("--version"));
}
