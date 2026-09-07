//! Coordinates committing only the staged files selected by the user.

use std::path::Path;

use anyhow::{Context, Result};

use crate::{git, staging::StagedFile};

pub struct CommitWorkflow<'a> {
    working_directory: &'a Path,
}

impl<'a> CommitWorkflow<'a> {
    pub fn new(working_directory: &'a Path) -> Self {
        Self { working_directory }
    }

    pub fn commit(&self, message: &str, sign: bool, excluded_files: &[StagedFile]) -> Result<()> {
        let excluded_patch = if excluded_files.is_empty() {
            None
        } else {
            Some(git::staged_patch(self.working_directory, excluded_files)?)
        };

        if !excluded_files.is_empty() {
            git::unstage(self.working_directory, excluded_files)?;
        }

        if let Err(commit_error) = git::commit(self.working_directory, message, sign) {
            if let Some(excluded_patch) = excluded_patch {
                git::restore_staged_patch(self.working_directory, &excluded_patch).with_context(|| {
                    format!(
                        "git commit failed and restoring excluded paths to the index also failed: {commit_error:#}"
                    )
                })?;
                return Err(commit_error
                    .context("git commit failed; excluded paths were restored to the index"));
            }
            return Err(commit_error);
        }

        Ok(())
    }
}
