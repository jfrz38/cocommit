//! Git command execution and preflight checks.

use std::{
    io,
    path::Path,
    process::{Command, Output},
};

use anyhow::{Result, anyhow, bail};

/// Verifies that Git is available, the current directory is a work tree, and
/// the index contains staged changes.
pub fn preflight(working_directory: &Path) -> Result<()> {
    let output = Command::new("git")
        .current_dir(working_directory)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|error| git_start_error("git rev-parse --is-inside-work-tree", error))?;

    if !output.status.success() {
        return Err(git_failure("git rev-parse --is-inside-work-tree", &output));
    }
    if !is_inside_work_tree(true, &output.stdout) {
        bail!(
            "git rev-parse --is-inside-work-tree reported that the current directory is not inside a usable Git working tree"
        );
    }

    let output = Command::new("git")
        .current_dir(working_directory)
        .args(["diff", "--cached", "--quiet"])
        .output()
        .map_err(|error| git_start_error("git diff --cached --quiet", error))?;

    if !has_staged_changes(output.status.code())? {
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
        .map_err(|error| git_start_error("git commit", error))?;

    if status.success() {
        Ok(())
    } else {
        bail!("git commit failed with status {status}")
    }
}

fn git_start_error(command: &str, error: io::Error) -> anyhow::Error {
    if error.kind() == io::ErrorKind::NotFound {
        anyhow!(
            "Git executable was not found while running `{command}`; install Git and ensure it is on PATH"
        )
    } else {
        anyhow!("failed to start `{command}`: {error}")
    }
}

fn git_failure(command: &str, output: &Output) -> anyhow::Error {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        anyhow!("`{command}` failed with status {}", output.status)
    } else {
        anyhow!("`{command}` failed with status {}: {stderr}", output.status)
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
