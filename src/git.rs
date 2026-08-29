//! Git command execution and preflight checks.

use std::{path::Path, process::Command};

use anyhow::{Context, Result, bail};

/// Verifies that Git is available, the current directory is a work tree, and
/// the index contains staged changes.
pub fn preflight(working_directory: &Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(working_directory)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("failed to run git rev-parse; ensure Git is installed and on PATH")?;

    if !is_inside_work_tree(output.status.success(), &output.stdout) {
        bail!("current directory is not inside a usable Git working tree");
    }

    let status = Command::new("git")
        .current_dir(working_directory)
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("failed to run git diff; ensure Git is installed and on PATH")?;

    if !has_staged_changes(status.code())? {
        bail!("no staged changes to commit");
    }

    Ok(())
}

/// Runs `git commit` with the supplied message and optional explicit signing.
pub fn commit(working_directory: &Path, message: &str, sign: bool) -> Result<()> {
    let status = Command::new("git")
        .current_dir(working_directory)
        .args(commit_arguments(message, sign))
        .status()
        .context("failed to run git commit; ensure Git is installed and on PATH")?;

    if status.success() {
        Ok(())
    } else {
        bail!("git commit failed with status {status}")
    }
}

fn commit_arguments(message: &str, sign: bool) -> Vec<String> {
    let mut arguments = vec!["commit".to_owned()];
    if sign {
        arguments.push("-S".to_owned());
    }
    arguments.extend(["-m".to_owned(), message.to_owned()]);
    arguments
}

fn is_inside_work_tree(success: bool, stdout: &[u8]) -> bool {
    success && stdout.trim_ascii() == b"true"
}

fn has_staged_changes(status_code: Option<i32>) -> Result<bool> {
    match status_code {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        Some(code) => bail!("git diff --cached --quiet failed with exit code {code}"),
        None => bail!("git diff --cached --quiet was terminated by a signal"),
    }
}

#[cfg(test)]
mod tests;
