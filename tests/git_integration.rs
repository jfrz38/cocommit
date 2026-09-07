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
    run_git(repository.path(), ["config", "commit.gpgSign", "true"]);

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

    let message = cocommit::commit::CommitDraft::new(
        "feat".to_owned(),
        None,
        false,
        "verify Git integration".to_owned(),
        None,
        Some("Preserve quotes and Unicode: \"ready\" for Marta García.".to_owned()),
        vec![cocommit::commit::Footer::new(
            "Closes".to_owned(),
            "#42".to_owned(),
        )],
    )
    .validated_message()
    .expect("complete message should be valid");
    cocommit::git::commit(repository.path(), &message, false)
        .expect("production Git commit should succeed");

    let output = Command::new("git")
        .current_dir(repository.path())
        .args(["log", "-1", "--format=%B"])
        .output()
        .expect("git log should run");
    assert!(output.status.success(), "git log should succeed");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim_end_matches('\n'),
        message
    );

    let signature_status = Command::new("git")
        .current_dir(repository.path())
        .args(["log", "-1", "--format=%G?"])
        .output()
        .expect("git log signature status should run");
    assert!(signature_status.status.success(), "git log should succeed");
    assert_eq!(
        String::from_utf8_lossy(&signature_status.stdout).trim(),
        "N",
        "an unsigned choice must override commit.gpgSign"
    );
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
fn preflight_rejects_a_bare_repository() {
    if !require_git() {
        return;
    }
    let parent = tempdir().expect("temporary repository parent should be created");
    let repository = parent.path().join("repository.git");
    run_git(parent.path(), ["init", "--bare", "repository.git"]);

    let error = cocommit::git::preflight(&repository)
        .expect_err("a bare repository should not be accepted as a working tree");

    assert!(
        error
            .to_string()
            .contains("not inside a usable Git working tree")
    );
}

#[test]
fn hidden_staged_section_commits_every_staged_file_without_unstaging() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    configure_identity(repository.path());
    fs::write(repository.path().join("one.txt"), "one\n").expect("first file should be written");
    fs::write(repository.path().join("two.txt"), "two\n").expect("second file should be written");
    run_git(repository.path(), ["add", "."]);

    let mut app = cocommit::app::App::new(false).with_sections(cocommit::settings::UiSections {
        staged_changes: false,
        ..Default::default()
    });
    app.set_staged_changes(
        cocommit::git::preflight(repository.path()).expect("preflight should pass"),
    );
    for _ in 0..3 {
        app.handle(cocommit::app::AppEvent::Tab);
    }
    app.handle(cocommit::app::AppEvent::Paste(
        "commit all staged files".to_owned(),
    ));

    assert!(app.excluded_staged_files().is_empty());
    let draft = app.draft().expect("draft should be valid");
    cocommit::git::commit(repository.path(), &draft.render_message(), false)
        .expect("production Git commit should succeed");

    let output = Command::new("git")
        .current_dir(repository.path())
        .args(["show", "--format=", "--name-only", "HEAD"])
        .output()
        .expect("git show should run");
    assert!(output.status.success(), "git show should succeed");
    let files = String::from_utf8_lossy(&output.stdout);
    assert!(files.contains("one.txt"));
    assert!(files.contains("two.txt"));
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
fn resolves_the_work_tree_root_from_subdirectories_and_linked_worktrees() {
    if !require_git() {
        return;
    }
    let parent = tempdir().expect("temporary repository parent should be created");
    let repository = parent.path().join(" repository");
    fs::create_dir(&repository).expect("repository directory should be created");
    run_git(&repository, ["init"]);
    configure_identity(&repository);
    fs::write(repository.join("baseline.txt"), "baseline\n")
        .expect("baseline file should be written");
    run_git(&repository, ["add", "baseline.txt"]);
    run_git(&repository, ["commit", "-m", "chore: baseline"]);

    let nested = repository.join("nested");
    fs::create_dir(&nested).expect("nested directory should be created");
    assert_eq!(
        cocommit::git::work_tree_root(&nested)
            .expect("work-tree root should resolve")
            .canonicalize()
            .expect("resolved work-tree root should exist"),
        repository
            .canonicalize()
            .expect("repository directory should exist")
    );
    fs::write(repository.join("staged.txt"), "staged\n").expect("staged file should be written");
    run_git(&repository, ["add", "staged.txt"]);
    cocommit::git::preflight(&nested).expect("preflight should accept a nested directory");

    let linked_parent = tempdir().expect("linked worktree parent should be created");
    let linked = linked_parent.path().join("linked");
    run_git_slice(
        &repository,
        [
            "worktree",
            "add",
            "-b",
            "linked-worktree",
            linked
                .to_str()
                .expect("temporary path should be valid Unicode"),
        ],
    );
    assert_eq!(
        cocommit::git::work_tree_root(&linked)
            .expect("linked work-tree root should resolve")
            .canonicalize()
            .expect("resolved linked work-tree root should exist"),
        linked
            .canonicalize()
            .expect("linked work-tree directory should exist")
    );
    fs::write(linked.join("linked-staged.txt"), "staged\n")
        .expect("linked worktree file should be written");
    run_git(&linked, ["add", "linked-staged.txt"]);
    cocommit::git::preflight(&linked).expect("preflight should accept a linked worktree");
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
    assert!(summary.files.iter().any(|file| {
        file.display_path == "docs/usage-界.md" && file.previous_display_path.is_some()
    }));
}

#[test]
fn unstaging_each_change_preserves_the_working_tree_and_refreshes_the_summary() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    configure_identity(repository.path());
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
    fs::write(repository.path().join("added [literal].txt"), "added\n")
        .expect("added file should be written");
    run_git(repository.path(), ["mv", "docs/guide.md", "docs/usage.md"]);
    run_git(repository.path(), ["add", "-A"]);

    let summary =
        cocommit::git::staged_changes(repository.path()).expect("staged summary should be read");
    cocommit::git::unstage(repository.path(), &summary.files)
        .expect("selected files should unstage successfully");
    let summary =
        cocommit::git::staged_changes(repository.path()).expect("staged summary should be read");

    assert!(summary.files.is_empty());
    assert_eq!(
        fs::read_to_string(repository.path().join("modified.txt")).unwrap(),
        "after\n"
    );
    assert!(!repository.path().join("deleted.txt").exists());
    assert_eq!(
        fs::read_to_string(repository.path().join("added [literal].txt")).unwrap(),
        "added\n"
    );
    assert!(repository.path().join("docs/usage.md").exists());
}

#[test]
fn unstaging_an_initial_repository_file_keeps_it_in_the_working_tree() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    fs::write(repository.path().join("initial [file].txt"), "content\n")
        .expect("initial file should be written");
    run_git(repository.path(), ["add", "initial [file].txt"]);
    let file = cocommit::git::staged_changes(repository.path())
        .expect("staged summary should be read")
        .files
        .into_iter()
        .next()
        .expect("initial file should be staged");

    cocommit::git::unstage(repository.path(), &[file])
        .expect("initial repository file should unstage");
    let summary =
        cocommit::git::staged_changes(repository.path()).expect("staged summary should be read");

    assert!(summary.files.is_empty());
    assert_eq!(
        fs::read_to_string(repository.path().join("initial [file].txt")).unwrap(),
        "content\n"
    );
}

#[test]
fn failed_commit_restores_excluded_files_to_the_index_without_staging_new_worktree_edits() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    configure_identity(repository.path());
    fs::write(repository.path().join("included.txt"), "before\n")
        .expect("included baseline should be written");
    fs::write(repository.path().join("excluded.txt"), "before\n")
        .expect("excluded baseline should be written");
    run_git(repository.path(), ["add", "."]);
    run_git(repository.path(), ["commit", "-m", "chore: baseline"]);

    fs::write(repository.path().join("included.txt"), "included staged\n")
        .expect("included change should be written");
    fs::write(repository.path().join("excluded.txt"), "excluded staged\n")
        .expect("excluded change should be written");
    run_git(repository.path(), ["add", "."]);
    fs::write(
        repository.path().join("excluded.txt"),
        "excluded unstaged\n",
    )
    .expect("unstaged excluded change should be written");

    install_failing_pre_commit_hook(repository.path());

    let excluded = cocommit::git::staged_changes(repository.path())
        .expect("staged summary should be read")
        .files
        .into_iter()
        .find(|file| file.display_path == "excluded.txt")
        .expect("excluded file should be staged");
    let error = cocommit::commit_workflow::CommitWorkflow::new(repository.path())
        .commit("feat: restore excluded index entry", false, &[excluded])
        .expect_err("the pre-commit hook should reject the commit");

    assert!(
        error
            .to_string()
            .contains("excluded paths were restored to the index")
    );
    assert_eq!(
        git_output(repository.path(), ["show", ":excluded.txt"]),
        "excluded staged\n"
    );
    assert!(
        git_output(repository.path(), ["diff", "--cached", "--name-only"])
            .lines()
            .any(|path| path == "excluded.txt"),
        "the excluded change should remain staged after restoration"
    );
    assert_eq!(
        fs::read_to_string(repository.path().join("excluded.txt"))
            .expect("excluded working-tree file should remain readable"),
        "excluded unstaged\n"
    );
    assert_eq!(
        git_output(repository.path(), ["show", ":included.txt"]),
        "included staged\n"
    );
}

#[test]
fn failed_commit_restores_an_excluded_rename_without_changing_the_working_tree() {
    if !require_git() {
        return;
    }
    let repository = tempdir().expect("temporary repository should be created");
    run_git(repository.path(), ["init"]);
    configure_identity(repository.path());
    fs::write(repository.path().join("included.txt"), "before\n")
        .expect("included baseline should be written");
    fs::write(repository.path().join("original.txt"), "before\n")
        .expect("rename baseline should be written");
    run_git(repository.path(), ["add", "."]);
    run_git(repository.path(), ["commit", "-m", "chore: baseline"]);

    fs::write(repository.path().join("included.txt"), "included staged\n")
        .expect("included change should be written");
    fs::rename(
        repository.path().join("original.txt"),
        repository.path().join("renamed.txt"),
    )
    .expect("rename should succeed");
    run_git(repository.path(), ["add", "-A"]);
    fs::write(repository.path().join("renamed.txt"), "renamed unstaged\n")
        .expect("unstaged rename change should be written");

    install_failing_pre_commit_hook(repository.path());

    let excluded = cocommit::git::staged_changes(repository.path())
        .expect("staged summary should be read")
        .files
        .into_iter()
        .find(|file| file.display_path == "renamed.txt")
        .expect("renamed file should be staged");
    cocommit::commit_workflow::CommitWorkflow::new(repository.path())
        .commit("feat: restore excluded rename", false, &[excluded])
        .expect_err("the pre-commit hook should reject the commit");

    assert!(
        git_output(
            repository.path(),
            ["diff", "--cached", "--name-status", "--find-renames"],
        )
        .contains("R100\toriginal.txt\trenamed.txt")
    );
    assert_eq!(
        git_output(repository.path(), ["show", ":renamed.txt"]),
        "before\n"
    );
    assert!(!repository.path().join("original.txt").exists());
    assert_eq!(
        fs::read_to_string(repository.path().join("renamed.txt"))
            .expect("renamed working-tree file should remain readable"),
        "renamed unstaged\n"
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
    run_git_slice(repository, arguments);
}

fn run_git_slice<I, S>(repository: &Path, arguments: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("git")
        .current_dir(repository)
        .args(arguments)
        .status()
        .expect("Git command should run");
    assert!(status.success(), "Git command should succeed");
}

fn git_output<const N: usize>(repository: &Path, arguments: [&str; N]) -> String {
    let output = Command::new("git")
        .current_dir(repository)
        .args(arguments)
        .output()
        .expect("Git command should run");
    assert!(output.status.success(), "Git command should succeed");
    String::from_utf8(output.stdout).expect("Git output should be UTF-8")
}

fn install_failing_pre_commit_hook(repository: &Path) {
    let hooks_directory = repository.join("hooks");
    fs::create_dir(&hooks_directory).expect("hooks directory should be created");
    let hook = hooks_directory.join("pre-commit");
    fs::write(&hook, "#!/bin/sh\nexit 1\n").expect("failing hook should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&hook)
            .expect("failing hook metadata should be read")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions).expect("failing hook should be executable");
    }
    run_git(
        repository,
        [
            "config",
            "core.hooksPath",
            hooks_directory
                .to_str()
                .expect("temporary path should be valid Unicode"),
        ],
    );
}

fn configure_identity(repository: &Path) {
    run_git(
        repository,
        ["config", "user.name", "Cocommit Integration Test"],
    );
    run_git(
        repository,
        ["config", "user.email", "cocommit@example.test"],
    );
    run_git(repository, ["config", "commit.gpgSign", "false"]);
}
