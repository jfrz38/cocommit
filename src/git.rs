//! Git command execution and preflight checks.

use std::{
    ffi::OsString,
    fmt::Write,
    io,
    path::Path,
    process::{Command, Output},
};

use anyhow::{Context, Result, anyhow, bail};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StagedChanges {
    pub files: Vec<StagedFile>,
    pub insertions: u64,
    pub deletions: u64,
    pub binary_files: usize,
}

impl StagedChanges {
    pub fn status_summary(&self) -> String {
        let additions = self
            .files
            .iter()
            .filter(|file| file.kind.is_added())
            .count();
        let modifications = self
            .files
            .iter()
            .filter(|file| file.kind.is_modified())
            .count();
        let deletions = self
            .files
            .iter()
            .filter(|file| file.kind.is_deleted())
            .count();
        let renames = self
            .files
            .iter()
            .filter(|file| file.kind.is_renamed())
            .count();
        format!("A:{additions} M:{modifications} D:{deletions} R:{renames}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedFile {
    pub kind: StagedChangeKind,
    pub path: String,
    pub previous_path: Option<String>,
    git_paths: Vec<OsString>,
}

impl StagedFile {
    pub fn for_display(
        kind: StagedChangeKind,
        path: impl Into<String>,
        previous_path: Option<String>,
    ) -> Self {
        let path = path.into();
        let mut git_paths = previous_path
            .as_ref()
            .map(|previous_path| vec![OsString::from(previous_path)])
            .unwrap_or_default();
        git_paths.push(OsString::from(&path));
        Self {
            kind,
            path,
            previous_path,
            git_paths,
        }
    }

    pub fn label(&self) -> String {
        match &self.previous_path {
            Some(previous_path) => {
                format!("{} {previous_path} -> {}", self.kind.label(), self.path)
            }
            None => format!("{} {}", self.kind.label(), self.path),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StagedChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Other(String),
}

impl StagedChangeKind {
    fn from_status(status: &str) -> Self {
        match status.as_bytes().first() {
            Some(b'A') => Self::Added,
            Some(b'M') => Self::Modified,
            Some(b'D') => Self::Deleted,
            Some(b'R') => Self::Renamed,
            _ => Self::Other(status.to_owned()),
        }
    }

    fn label(&self) -> &str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Other(status) => status,
        }
    }

    fn is_added(&self) -> bool {
        matches!(self, Self::Added)
    }

    fn is_modified(&self) -> bool {
        matches!(self, Self::Modified)
    }

    fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted)
    }

    fn is_renamed(&self) -> bool {
        matches!(self, Self::Renamed)
    }
}

/// Verifies that Git is available, the current directory is a work tree, and
/// the index contains staged changes.
pub fn preflight(working_directory: &Path) -> Result<StagedChanges> {
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

    staged_changes(working_directory)
}

/// Reads a display-safe, read-only snapshot of the staged Git index.
pub fn staged_changes(working_directory: &Path) -> Result<StagedChanges> {
    let name_status = Command::new("git")
        .current_dir(working_directory)
        .args(["diff", "--cached", "--name-status", "-z", "--find-renames"])
        .output()
        .map_err(|error| {
            git_start_error("git diff --cached --name-status -z --find-renames", error)
        })?;
    if !name_status.status.success() {
        return Err(git_failure(
            "git diff --cached --name-status -z --find-renames",
            &name_status,
        ));
    }

    let numstat = Command::new("git")
        .current_dir(working_directory)
        .args(["diff", "--cached", "--numstat", "-z", "--find-renames"])
        .output()
        .map_err(|error| git_start_error("git diff --cached --numstat -z --find-renames", error))?;
    if !numstat.status.success() {
        return Err(git_failure(
            "git diff --cached --numstat -z --find-renames",
            &numstat,
        ));
    }

    let (insertions, deletions, binary_files) = parse_numstat(&numstat.stdout)?;
    Ok(StagedChanges {
        files: parse_name_status(&name_status.stdout)?,
        insertions,
        deletions,
        binary_files,
    })
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

/// Removes selected files from the index without changing the working tree.
pub fn unstage(working_directory: &Path, files: &[StagedFile]) -> Result<()> {
    let has_head = has_head(working_directory)?;
    let arguments = unstage_arguments(files, has_head);
    let output = Command::new("git")
        .current_dir(working_directory)
        .args(arguments)
        .output()
        .map_err(|error| git_start_error("git unstage selected file", error))?;
    if !output.status.success() {
        return Err(git_failure("git unstage selected file", &output));
    }

    Ok(())
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

fn has_head(working_directory: &Path) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(working_directory)
        .args(["rev-parse", "--verify", "--quiet", "HEAD"])
        .output()
        .map_err(|error| git_start_error("git rev-parse --verify --quiet HEAD", error))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(code) => bail!("git rev-parse --verify --quiet HEAD failed with exit code {code}"),
        None => bail!("git rev-parse --verify --quiet HEAD was terminated by a signal"),
    }
}

fn unstage_arguments(files: &[StagedFile], has_head: bool) -> Vec<OsString> {
    let mut arguments = if has_head {
        vec![
            OsString::from("--literal-pathspecs"),
            OsString::from("restore"),
            OsString::from("--staged"),
        ]
    } else {
        vec![
            OsString::from("--literal-pathspecs"),
            OsString::from("update-index"),
            OsString::from("--force-remove"),
        ]
    };
    arguments.push(OsString::from("--"));
    arguments.extend(files.iter().flat_map(|file| file.git_paths.iter().cloned()));
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

fn parse_name_status(output: &[u8]) -> Result<Vec<StagedFile>> {
    let mut fields = output
        .split(|byte| *byte == b'\0')
        .filter(|field| !field.is_empty());
    let mut files = Vec::new();

    while let Some(status) = fields.next() {
        let status =
            std::str::from_utf8(status).context("Git returned a non-UTF-8 change status")?;
        let kind = StagedChangeKind::from_status(status);
        let first_path = fields
            .next()
            .context("Git returned a staged change without a path")?;
        let (previous_path, path, git_paths) = if matches!(kind, StagedChangeKind::Renamed) {
            let path = fields
                .next()
                .context("Git returned a staged rename without a destination path")?;
            (
                Some(display_path(first_path)),
                display_path(path),
                vec![path_from_git(first_path), path_from_git(path)],
            )
        } else {
            (
                None,
                display_path(first_path),
                vec![path_from_git(first_path)],
            )
        };
        files.push(StagedFile {
            kind,
            path,
            previous_path,
            git_paths,
        });
    }

    Ok(files)
}

fn parse_numstat(output: &[u8]) -> Result<(u64, u64, usize)> {
    let fields = output.split(|byte| *byte == b'\0').collect::<Vec<_>>();
    let mut index = 0;
    let mut insertions = 0;
    let mut deletions = 0;
    let mut binary_files = 0;

    while index < fields.len() {
        let record = fields[index];
        index += 1;
        if record.is_empty() {
            continue;
        }
        let mut parts = record.splitn(3, |byte| *byte == b'\t');
        let added = parts
            .next()
            .context("Git returned an incomplete numstat record")?;
        let deleted = parts
            .next()
            .context("Git returned an incomplete numstat record")?;
        let path = parts
            .next()
            .context("Git returned an incomplete numstat record")?;

        if path.is_empty() {
            // Renames use two extra NUL-delimited path fields.
            index += 2;
            if index > fields.len() {
                bail!("Git returned an incomplete renamed-file numstat record");
            }
        }
        if added == b"-" && deleted == b"-" {
            binary_files += 1;
            continue;
        }
        insertions += parse_stat_count(added)?;
        deletions += parse_stat_count(deleted)?;
    }

    Ok((insertions, deletions, binary_files))
}

fn parse_stat_count(value: &[u8]) -> Result<u64> {
    std::str::from_utf8(value)
        .context("Git returned a non-UTF-8 numstat count")?
        .parse()
        .context("Git returned an invalid numstat count")
}

fn display_path(path: &[u8]) -> String {
    let mut displayed = String::new();
    for character in String::from_utf8_lossy(path).chars() {
        match character {
            '\n' => displayed.push_str("\\n"),
            '\r' => displayed.push_str("\\r"),
            '\t' => displayed.push_str("\\t"),
            character if character.is_control() => {
                write!(displayed, "\\u{{{:x}}}", character as u32)
                    .expect("writing to a String cannot fail");
            }
            character => displayed.push(character),
        }
    }
    displayed
}

#[cfg(unix)]
fn path_from_git(path: &[u8]) -> OsString {
    use std::os::unix::ffi::OsStringExt;

    OsString::from_vec(path.to_vec())
}

#[cfg(not(unix))]
fn path_from_git(path: &[u8]) -> OsString {
    OsString::from(String::from_utf8_lossy(path).into_owned())
}

#[cfg(test)]
mod tests;
