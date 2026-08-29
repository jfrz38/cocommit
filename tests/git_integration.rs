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

fn run_git<const N: usize>(repository: &Path, arguments: [&str; N]) {
    let status = Command::new("git")
        .current_dir(repository)
        .args(arguments)
        .status()
        .expect("Git command should run");
    assert!(status.success(), "Git command should succeed");
}
