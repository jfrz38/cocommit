use std::io;

use super::{
    StagedChangeKind, commit_arguments, git_start_error, has_staged_changes, is_inside_work_tree,
    parse_name_status, parse_numstat,
};

#[test]
fn builds_unsigned_commit_arguments() {
    assert_eq!(
        commit_arguments("feat: add Git commands", false),
        ["commit", "-m", "feat: add Git commands"]
    );
}

#[test]
fn adds_signing_flag_only_when_requested() {
    let arguments = commit_arguments("fix: handle error", true);

    assert_eq!(arguments, ["commit", "-S", "-m", "fix: handle error"]);
    assert!(!arguments.contains(&"--no-gpg-sign".to_owned()));
}

#[test]
fn keeps_message_with_spaces_and_quotes_as_one_argument() {
    let message = "fix: preserve \"quoted text\"";
    let arguments = commit_arguments(message, false);

    assert_eq!(arguments.len(), 3);
    assert_eq!(arguments[2], message);
}

#[test]
fn recognizes_successful_work_tree_check_with_true_output() {
    assert!(is_inside_work_tree(true, b"true\n"));
}

#[test]
fn rejects_failed_or_non_work_tree_checks() {
    assert!(!is_inside_work_tree(false, b"true\n"));
    assert!(!is_inside_work_tree(true, b"false\n"));
}

#[test]
fn interprets_preflight_exit_codes() {
    assert!(!has_staged_changes(Some(0)).expect("exit code 0 should be accepted"));
    assert!(has_staged_changes(Some(1)).expect("exit code 1 should be accepted"));
}

#[test]
fn rejects_unexpected_preflight_exit_codes() {
    let error = has_staged_changes(Some(2)).expect_err("exit code 2 should fail preflight");

    assert!(error.to_string().contains("exit code 2"));
}

#[test]
fn rejects_preflight_termination_by_signal() {
    let error = has_staged_changes(None).expect_err("signal termination should fail preflight");

    assert!(error.to_string().contains("terminated by a signal"));
}

#[test]
fn identifies_a_missing_git_executable() {
    let error = git_start_error(
        "git rev-parse --is-inside-work-tree",
        io::Error::new(io::ErrorKind::NotFound, "not found"),
    );

    assert!(error.to_string().contains("Git executable was not found"));
    assert!(
        error
            .to_string()
            .contains("git rev-parse --is-inside-work-tree")
    );
}

#[test]
fn identifies_other_command_start_failures() {
    let error = git_start_error(
        "git diff --cached --quiet",
        io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
    );

    assert!(error.to_string().contains("failed to start"));
    assert!(error.to_string().contains("access denied"));
}

#[test]
fn parses_nul_delimited_staged_statuses_without_splitting_paths() {
    let files = parse_name_status(
        b"A\0added file.txt\0M\0src/main.rs\0D\0deleted.txt\0R100\0old\tname\0new\nname\0",
    )
    .expect("name-status output should parse");

    assert_eq!(files.len(), 4);
    assert_eq!(files[0].kind, StagedChangeKind::Added);
    assert_eq!(files[0].path, "added file.txt");
    assert_eq!(files[1].kind, StagedChangeKind::Modified);
    assert_eq!(files[2].kind, StagedChangeKind::Deleted);
    assert_eq!(files[3].kind, StagedChangeKind::Renamed);
    assert_eq!(files[3].previous_path.as_deref(), Some("old\\tname"));
    assert_eq!(files[3].path, "new\\nname");
}

#[test]
fn parses_numstat_totals_for_text_binary_and_renamed_files() {
    let (insertions, deletions, binary_files) =
        parse_numstat(b"2\t1\tadded.txt\0-\t-\timage.png\x003\t4\t\0old.txt\0new.txt\0")
            .expect("numstat output should parse");

    assert_eq!((insertions, deletions, binary_files), (5, 5, 1));
}

#[test]
fn rejects_incomplete_renamed_file_records() {
    let error = parse_name_status(b"R100\0old.txt\0")
        .expect_err("a rename without a destination should fail");

    assert!(error.to_string().contains("destination path"));
}
