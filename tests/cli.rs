use std::{process::Command, str};

use tempfile::tempdir;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cocommit"))
}

#[test]
fn help_works_outside_a_repository_with_redirected_output() {
    let directory = tempdir().expect("temporary directory should be created");
    let output = binary()
        .current_dir(directory.path())
        .arg("--help")
        .output()
        .expect("help command should run");

    assert!(output.status.success());
    assert!(
        str::from_utf8(&output.stdout)
            .expect("help should be UTF-8")
            .contains("Usage: cocommit")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn version_works_outside_a_repository_with_redirected_output() {
    let directory = tempdir().expect("temporary directory should be created");
    let output = binary()
        .current_dir(directory.path())
        .arg("--version")
        .output()
        .expect("version command should run");

    assert!(output.status.success());
    assert_eq!(
        str::from_utf8(&output.stdout).expect("version should be UTF-8"),
        concat!("cocommit ", env!("CARGO_PKG_VERSION"), "\n")
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn unknown_arguments_return_a_usage_error_without_terminal_sequences() {
    let output = binary()
        .arg("--unknown")
        .output()
        .expect("command should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = str::from_utf8(&output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("unrecognized argument: --unknown"));
    assert!(stderr.contains("Usage: cocommit"));
    assert!(!stderr.contains('\u{1b}'));
}

#[test]
fn redirected_standard_streams_fail_before_git_or_terminal_initialization() {
    let output = binary().output().expect("command should run");

    assert_eq!(output.status.code(), Some(1));
    let stderr = str::from_utf8(&output.stderr).expect("error output should be UTF-8");
    assert!(stderr.contains("standard input and standard output must be interactive terminals"));
    assert!(!stderr.contains('\u{1b}'));
}
