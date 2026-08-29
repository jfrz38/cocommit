//! Conventional Commit domain model and validation.

/// A normalized Conventional Commit header ready to render or submit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDraft {
    pub commit_type: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub message: String,
    pub issue: Option<u64>,
}

/// A field in a commit draft that can fail validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftField {
    CommitType,
    Scope,
    Message,
    Issue,
}

/// The reason a draft field is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorKind {
    Required,
    MustBeSingleLine,
    ContainsWhitespace,
    ContainsForbiddenCharacter,
    InvalidDecimal,
    IntegerOutOfRange,
}

/// A validation error associated with one editable draft field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationError {
    pub field: DraftField,
    pub kind: ValidationErrorKind,
}

impl CommitDraft {
    /// Creates a normalized draft from already parsed values.
    pub fn new(
        commit_type: String,
        scope: Option<String>,
        breaking: bool,
        message: String,
        issue: Option<u64>,
    ) -> Self {
        Self {
            commit_type: normalize(&commit_type),
            scope: scope.and_then(|scope| non_empty_normalized(&scope)),
            breaking,
            message: normalize(&message),
            issue,
        }
    }

    /// Creates a draft from the form's raw text values.
    pub fn from_raw(
        commit_type: String,
        scope: Option<String>,
        breaking: bool,
        message: String,
        issue: Option<String>,
    ) -> Result<Self, Vec<ValidationError>> {
        let issue_result = issue.as_deref().map_or(Ok(None), Self::parse_issue);
        let draft = Self::new(commit_type, scope, breaking, message, None);
        let mut errors = draft.validate().err().unwrap_or_default();

        match issue_result {
            Ok(issue) => {
                let draft = Self { issue, ..draft };
                if errors.is_empty() {
                    Ok(draft)
                } else {
                    Err(errors)
                }
            }
            Err(error) => {
                errors.push(error);
                Err(errors)
            }
        }
    }

    /// Parses the optional issue identifier from its editable text representation.
    pub fn parse_issue(raw: &str) -> Result<Option<u64>, ValidationError> {
        let issue = normalize(raw);
        if issue.is_empty() {
            return Ok(None);
        }

        if !issue.bytes().all(|character| character.is_ascii_digit()) {
            return Err(validation_error(
                DraftField::Issue,
                ValidationErrorKind::InvalidDecimal,
            ));
        }

        issue.parse().map(Some).map_err(|_| {
            validation_error(DraftField::Issue, ValidationErrorKind::IntegerOutOfRange)
        })
    }

    /// Renders the canonical Conventional Commit header.
    pub fn render_message(&self) -> String {
        let mut message = self.commit_type.clone();

        if let Some(scope) = &self.scope {
            message.push('(');
            message.push_str(scope);
            message.push(')');
        }

        if self.breaking {
            message.push('!');
        }

        message.push_str(": ");
        message.push_str(&self.message);

        if let Some(issue) = self.issue {
            message.push_str(" (#");
            message.push_str(&issue.to_string());
            message.push(')');
        }

        message
    }

    /// Validates every field in a deterministic form order.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if normalize(&self.commit_type).is_empty() {
            errors.push(validation_error(
                DraftField::CommitType,
                ValidationErrorKind::Required,
            ));
        } else if contains_line_break(&self.commit_type) {
            errors.push(validation_error(
                DraftField::CommitType,
                ValidationErrorKind::MustBeSingleLine,
            ));
        } else if self.commit_type.chars().any(char::is_whitespace) {
            errors.push(validation_error(
                DraftField::CommitType,
                ValidationErrorKind::ContainsWhitespace,
            ));
        } else if self
            .commit_type
            .chars()
            .any(|character| matches!(character, '(' | ')' | '!' | ':'))
        {
            errors.push(validation_error(
                DraftField::CommitType,
                ValidationErrorKind::ContainsForbiddenCharacter,
            ));
        }

        if let Some(scope) = &self.scope {
            if contains_line_break(scope) {
                errors.push(validation_error(
                    DraftField::Scope,
                    ValidationErrorKind::MustBeSingleLine,
                ));
            } else if scope.contains(['(', ')']) {
                errors.push(validation_error(
                    DraftField::Scope,
                    ValidationErrorKind::ContainsForbiddenCharacter,
                ));
            }
        }

        if normalize(&self.message).is_empty() {
            errors.push(validation_error(
                DraftField::Message,
                ValidationErrorKind::Required,
            ));
        } else if contains_line_break(&self.message) {
            errors.push(validation_error(
                DraftField::Message,
                ValidationErrorKind::MustBeSingleLine,
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validates the draft before returning its canonical header.
    pub fn validated_message(&self) -> Result<String, Vec<ValidationError>> {
        self.validate()?;
        Ok(self.render_message())
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_owned()
}

fn non_empty_normalized(value: &str) -> Option<String> {
    let value = normalize(value);
    (!value.is_empty()).then_some(value)
}

fn contains_line_break(value: &str) -> bool {
    value.contains(['\n', '\r'])
}

fn validation_error(field: DraftField, kind: ValidationErrorKind) -> ValidationError {
    ValidationError { field, kind }
}

#[cfg(test)]
mod tests;
