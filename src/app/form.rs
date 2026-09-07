//! Editable commit fields.

use tui_input::Input;

use crate::{
    commit::{CommitDraft, Footer, MessagePolicy, ValidationError},
    settings::UiSections,
};

#[derive(Debug, Clone)]
pub struct FormState {
    pub commit_type: Input,
    pub scope: Input,
    pub breaking: bool,
    pub message: Input,
    pub body: Input,
    pub issue: Input,
    pub footers: Vec<Footer>,
}

impl FormState {
    /// Projects the editable fields into a normalized draft with the visible sections.
    pub(super) fn project(&self, sections: UiSections, issue: Option<u64>) -> CommitDraft {
        CommitDraft::new(
            self.commit_type.to_string(),
            Some(self.scope.to_string()),
            self.breaking,
            self.message.to_string(),
            issue,
            sections.body.then(|| self.body.to_string()),
            if sections.footers {
                self.footers.clone()
            } else {
                Vec::new()
            },
        )
    }

    pub(super) fn draft(
        &self,
        sections: UiSections,
        policy: &MessagePolicy,
    ) -> Result<CommitDraft, Vec<ValidationError>> {
        let issue = sections
            .issue
            .then(|| CommitDraft::parse_issue(&self.issue.to_string()))
            .transpose()
            .map(Option::flatten);
        let draft = self.project(sections, issue.as_ref().ok().copied().flatten());
        let mut errors = draft.validate().err().unwrap_or_default();

        if let Err(error) = issue {
            errors.push(error);
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        draft.validate_with_policy(policy)?;
        Ok(draft)
    }
}
