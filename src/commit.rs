//! Conventional Commit domain model and validation.

/// A normalized Conventional Commit message ready to render or submit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDraft {
    pub commit_type: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub message: String,
    pub issue: Option<u64>,
    pub body: Option<String>,
    pub footers: Vec<Footer>,
}

/// An ordered Conventional Commit footer or Git trailer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footer {
    pub token: String,
    pub value: String,
}

impl Footer {
    /// Creates a normalized footer from its token and value.
    pub fn new(token: String, value: String) -> Self {
        let token = normalize(&token);
        Self {
            token: if is_breaking_token(&token) {
                "BREAKING CHANGE".to_owned()
            } else {
                token
            },
            value: normalize_multiline(&value),
        }
    }

    /// Returns whether this footer describes a breaking change.
    pub fn is_breaking_change(&self) -> bool {
        self.token == "BREAKING CHANGE"
    }
}

/// A field in a commit draft that can fail validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftField {
    CommitType,
    Scope,
    Message,
    Issue,
    Footer(usize),
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
    ContainsBlankLine,
    DuplicateSemanticField,
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
        body: Option<String>,
        footers: Vec<Footer>,
    ) -> Self {
        Self {
            commit_type: normalize(&commit_type),
            scope: scope.and_then(|scope| non_empty_normalized(&scope)),
            breaking,
            message: normalize(&message),
            issue,
            body: body.and_then(|body| non_empty_multiline_normalized(&body)),
            footers,
        }
    }

    /// Creates a draft from the form's raw text values.
    pub fn from_raw(
        commit_type: String,
        scope: Option<String>,
        breaking: bool,
        message: String,
        issue: Option<String>,
        body: Option<String>,
        footers: Vec<Footer>,
    ) -> Result<Self, Vec<ValidationError>> {
        let issue_result = issue.as_deref().map_or(Ok(None), Self::parse_issue);
        let draft = Self::new(commit_type, scope, breaking, message, None, body, footers);
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

    /// Renders the canonical Conventional Commit message.
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

        if let Some(body) = &self.body {
            message.push_str("\n\n");
            message.push_str(body);
        }

        if !self.footers.is_empty() {
            message.push_str("\n\n");
            for (index, footer) in self.footers.iter().enumerate() {
                if index > 0 {
                    message.push('\n');
                }
                message.push_str(&footer.token);
                message.push_str(": ");
                message.push_str(&footer.value);
            }
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

        let mut has_breaking_footer = false;
        for (index, footer) in self.footers.iter().enumerate() {
            if !is_valid_footer_token(&footer.token) {
                errors.push(validation_error(
                    DraftField::Footer(index),
                    ValidationErrorKind::ContainsForbiddenCharacter,
                ));
            }
            if footer.value.trim().is_empty() {
                errors.push(validation_error(
                    DraftField::Footer(index),
                    ValidationErrorKind::Required,
                ));
            } else if contains_blank_line(&footer.value) {
                errors.push(validation_error(
                    DraftField::Footer(index),
                    ValidationErrorKind::ContainsBlankLine,
                ));
            }
            if footer.is_breaking_change() {
                if has_breaking_footer {
                    errors.push(validation_error(
                        DraftField::Footer(index),
                        ValidationErrorKind::DuplicateSemanticField,
                    ));
                }
                has_breaking_footer = true;
            }
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

fn normalize_multiline(value: &str) -> String {
    value.trim().to_owned()
}

fn non_empty_multiline_normalized(value: &str) -> Option<String> {
    let value = normalize_multiline(value);
    (!value.is_empty()).then_some(value)
}

fn contains_line_break(value: &str) -> bool {
    value.contains(['\n', '\r'])
}

fn contains_blank_line(value: &str) -> bool {
    value.lines().any(|line| line.trim().is_empty())
}

fn is_breaking_token(token: &str) -> bool {
    matches!(token, "BREAKING CHANGE" | "BREAKING-CHANGE")
}

fn is_valid_footer_token(token: &str) -> bool {
    is_breaking_token(token)
        || (!token.is_empty()
            && token.split('-').all(|segment| {
                !segment.is_empty()
                    && segment
                        .bytes()
                        .all(|character| character.is_ascii_alphanumeric())
            }))
}

fn validation_error(field: DraftField, kind: ValidationErrorKind) -> ValidationError {
    ValidationError { field, kind }
}

#[cfg(test)]
mod tests;
