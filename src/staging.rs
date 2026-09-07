//! Application-facing snapshot of staged Git changes.

use std::ffi::OsString;

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
    pub display_path: String,
    pub previous_display_path: Option<String>,
    pathspecs: Vec<OsString>,
}

impl StagedFile {
    /// Builds a visible staged file for application and UI tests.
    #[cfg(test)]
    pub(crate) fn from_display_paths(
        kind: StagedChangeKind,
        display_path: impl Into<String>,
        previous_display_path: Option<String>,
    ) -> Self {
        let display_path = display_path.into();
        let mut pathspecs = previous_display_path
            .as_ref()
            .map(|previous_path| vec![OsString::from(previous_path)])
            .unwrap_or_default();
        pathspecs.push(OsString::from(&display_path));
        Self {
            kind,
            display_path,
            previous_display_path,
            pathspecs,
        }
    }

    pub(crate) fn from_git_paths(
        kind: StagedChangeKind,
        display_path: String,
        previous_display_path: Option<String>,
        pathspecs: Vec<OsString>,
    ) -> Self {
        Self {
            kind,
            display_path,
            previous_display_path,
            pathspecs,
        }
    }

    pub(crate) fn pathspecs(&self) -> &[OsString] {
        &self.pathspecs
    }

    pub fn label(&self) -> String {
        match &self.previous_display_path {
            Some(previous_path) => {
                format!(
                    "{} {previous_path} -> {}",
                    self.kind.label(),
                    self.display_path
                )
            }
            None => format!("{} {}", self.kind.label(), self.display_path),
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
    pub(crate) fn from_status(status: &str) -> Self {
        match status.as_bytes().first() {
            Some(b'A') => Self::Added,
            Some(b'M') => Self::Modified,
            Some(b'D') => Self::Deleted,
            Some(b'R') => Self::Renamed,
            _ => Self::Other(status.to_owned()),
        }
    }

    pub fn label(&self) -> &str {
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
