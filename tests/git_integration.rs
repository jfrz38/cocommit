use std::{fs, path::Path, process::Command};

use tempfile::tempdir;

#[test]
fn creates_an_unsigned_commit_in_a_temporary_repository() {
    let Ok(version) = Command::new("git").arg("--version").status() else {
        eprintln!("skipping Git integration test because git --version could not run");
        return;
    };
    if !version.success() {
        eprintln!("skipping Git integration test because git --version failed with {version}");
        return;
    }

    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    run_git(
        repository.path(),
        ["config", "user.name", "Cocommit Integration Test"],
    );
    run_git(
        repository.path(),
        ["config", "user.email", "cocommit@example.test"],
    );
    run_git(repository.path(), ["config", "commit.gpgSign", "false"]);

    let hooks_directory = repository.path().join("empty-hooks");
    fs::create_dir(&hooks_directory).expect("empty hooks directory should be created");
    run_git(
        repository.path(),
        [
            "config",
            "core.hooksPath",
            hooks_directory
                .to_str()
                .expect("temporary path should be valid Unicode"),
        ],
    );

    fs::write(
        repository.path().join("staged.txt"),
        "integration coverage\n",
    )
    .expect("staged file should be written");
    run_git(repository.path(), ["add", "staged.txt"]);

    let message = "feat: verify Git integration";
    cocommit::git::commit(repository.path(), message, false)
        .expect("production Git commit should succeed");

    let output = Command::new("git")
        .current_dir(repository.path())
        .args(["log", "-1", "--format=%s"])
        .output()
        .expect("git log should run");
    assert!(output.status.success(), "git log should succeed");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), message);
}

#[test]
fn preflight_accepts_a_staged_repository_and_rejects_missing_staged_changes() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);

    let error = cocommit::git::preflight(repository.path())
        .expect_err("an empty index should fail preflight");
    assert!(error.to_string().contains("no staged changes"));

    fs::write(
        repository.path().join("staged.txt"),
        "integration coverage\n",
    )
    .expect("staged file should be written");
    run_git(repository.path(), ["add", "staged.txt"]);
    cocommit::git::preflight(repository.path()).expect("a staged repository should pass preflight");
}

#[test]
fn preflight_reports_the_failing_git_command_outside_a_repository() {
    if !require_git() {
        return;
    }
    let directory = tempdir().expect("temporary directory should be created");

    let error = cocommit::git::preflight(directory.path())
        .expect_err("a non-repository should fail preflight");

    assert!(
        error
            .to_string()
            .contains("git rev-parse --is-inside-work-tree")
    );
}

#[test]
fn staged_change_summary_matches_git_for_common_index_changes() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    run_git(
        repository.path(),
        ["config", "user.name", "Cocommit Integration Test"],
    );
    run_git(
        repository.path(),
        ["config", "user.email", "cocommit@example.test"],
    );
    run_git(repository.path(), ["config", "commit.gpgSign", "false"]);
    fs::create_dir(repository.path().join("docs")).expect("docs directory should be created");
    fs::write(repository.path().join("modified.txt"), "before\n")
        .expect("modified file should be written");
    fs::write(repository.path().join("deleted.txt"), "remove\n")
        .expect("deleted file should be written");
    fs::write(repository.path().join("docs/guide.md"), "guide\n").expect("guide should be written");
    run_git(repository.path(), ["add", "."]);
    run_git(repository.path(), ["commit", "-m", "chore: baseline"]);

    fs::write(repository.path().join("modified.txt"), "after\n")
        .expect("modified file should be updated");
    fs::remove_file(repository.path().join("deleted.txt")).expect("deleted file should be removed");
    fs::write(repository.path().join("added-界.txt"), "added\n")
        .expect("added file should be written");
    run_git(
        repository.path(),
        ["mv", "docs/guide.md", "docs/usage-界.md"],
    );
    run_git(repository.path(), ["add", "-A"]);

    let summary = cocommit::git::preflight(repository.path())
        .expect("staged repository summary should be read");

    assert_eq!(summary.files.len(), 4);
    assert_eq!(summary.status_summary(), "A:1 M:1 D:1 R:1");
    assert_eq!((summary.insertions, summary.deletions), (2, 2));
    assert!(
        summary
            .files
            .iter()
            .any(|file| file.path == "docs/usage-界.md" && file.previous_path.is_some())
    );
}

fn require_git() -> bool {
    let Ok(version) = Command::new("git").arg("--version").status() else {
        eprintln!("skipping Git integration test because git --version could not run");
        return false;
    };
    if !version.success() {
        eprintln!("skipping Git integration test because git --version failed with {version}");
        return false;
    }
    true
}

fn run_git<const N: usize>(repository: &Path, arguments: [&str; N]) {
    let status = Command::new("git")
        .current_dir(repository)
        .args(arguments)
        .status()
        .expect("Git command should run");
    assert!(status.success(), "Git command should succeed");
}
