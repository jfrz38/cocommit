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
mod tests {
    use super::{CommitDraft, DraftField, ValidationError, ValidationErrorKind};

    fn draft(
        commit_type: &str,
        scope: Option<&str>,
        breaking: bool,
        message: &str,
        issue: Option<u64>,
    ) -> CommitDraft {
        CommitDraft::new(
            commit_type.to_owned(),
            scope.map(str::to_owned),
            breaking,
            message.to_owned(),
            issue,
        )
    }

    fn error(field: DraftField, kind: ValidationErrorKind) -> ValidationError {
        ValidationError { field, kind }
    }

    #[test]
    fn renders_all_optional_field_combinations() {
        let cases = [
            (None, false, None, "feat: add authentication"),
            (Some("api"), false, None, "feat(api): add authentication"),
            (None, true, None, "feat!: add authentication"),
            (Some("api"), true, None, "feat(api)!: add authentication"),
            (None, false, Some(123), "feat: add authentication (#123)"),
            (
                Some("api"),
                false,
                Some(123),
                "feat(api): add authentication (#123)",
            ),
            (None, true, Some(123), "feat!: add authentication (#123)"),
            (
                Some("api"),
                true,
                Some(123),
                "feat(api)!: add authentication (#123)",
            ),
        ];

        for (scope, breaking, issue, expected) in cases {
            assert_eq!(
                draft("feat", scope, breaking, "add authentication", issue).render_message(),
                expected
            );
        }
    }

    #[test]
    fn normalizes_outer_whitespace_and_empty_scope() {
        let draft = draft(" feat ", Some("  "), false, " add authentication ", None);

        assert_eq!(draft.commit_type, "feat");
        assert_eq!(draft.scope, None);
        assert_eq!(draft.message, "add authentication");
        assert_eq!(draft.render_message(), "feat: add authentication");
    }

    #[test]
    fn renders_custom_type() {
        assert_eq!(
            draft("release", None, false, "publish version", None).render_message(),
            "release: publish version"
        );
    }

    #[test]
    fn rejects_invalid_type_values() {
        let cases = [
            ("", ValidationErrorKind::Required),
            ("   ", ValidationErrorKind::Required),
            ("feature type", ValidationErrorKind::ContainsWhitespace),
            ("feature\ntype", ValidationErrorKind::MustBeSingleLine),
            (
                "feature(type",
                ValidationErrorKind::ContainsForbiddenCharacter,
            ),
            ("feature)", ValidationErrorKind::ContainsForbiddenCharacter),
            ("feature!", ValidationErrorKind::ContainsForbiddenCharacter),
            ("feature:", ValidationErrorKind::ContainsForbiddenCharacter),
        ];

        for (commit_type, kind) in cases {
            assert_eq!(
                draft(commit_type, None, false, "message", None).validate(),
                Err(vec![error(DraftField::CommitType, kind)])
            );
        }
    }

    #[test]
    fn rejects_invalid_scope_values() {
        let cases = [
            ("api\nclient", ValidationErrorKind::MustBeSingleLine),
            (
                "api(client",
                ValidationErrorKind::ContainsForbiddenCharacter,
            ),
            ("api)", ValidationErrorKind::ContainsForbiddenCharacter),
        ];

        for (scope, kind) in cases {
            assert_eq!(
                draft("feat", Some(scope), false, "message", None).validate(),
                Err(vec![error(DraftField::Scope, kind)])
            );
        }
    }

    #[test]
    fn rejects_empty_or_multiline_message() {
        for (message, kind) in [
            ("", ValidationErrorKind::Required),
            ("   ", ValidationErrorKind::Required),
            ("first\nsecond", ValidationErrorKind::MustBeSingleLine),
        ] {
            assert_eq!(
                draft("feat", None, false, message, None).validate(),
                Err(vec![error(DraftField::Message, kind)])
            );
        }
    }

    #[test]
    fn parses_optional_issue() {
        assert_eq!(CommitDraft::parse_issue("  "), Ok(None));
        assert_eq!(CommitDraft::parse_issue(" 123 "), Ok(Some(123)));
        assert_eq!(CommitDraft::parse_issue("001"), Ok(Some(1)));
    }

    #[test]
    fn rejects_non_decimal_or_overflowing_issue() {
        assert_eq!(
            CommitDraft::parse_issue("12a"),
            Err(error(
                DraftField::Issue,
                ValidationErrorKind::InvalidDecimal
            ))
        );
        assert_eq!(
            CommitDraft::parse_issue("18446744073709551616"),
            Err(error(
                DraftField::Issue,
                ValidationErrorKind::IntegerOutOfRange
            ))
        );
    }

    #[test]
    fn from_raw_aggregates_errors_in_field_order() {
        assert_eq!(
            CommitDraft::from_raw(
                "bad type".to_owned(),
                Some("bad(scope".to_owned()),
                false,
                "\n".to_owned(),
                Some("too-big".to_owned()),
            ),
            Err(vec![
                error(
                    DraftField::CommitType,
                    ValidationErrorKind::ContainsWhitespace
                ),
                error(
                    DraftField::Scope,
                    ValidationErrorKind::ContainsForbiddenCharacter
                ),
                error(DraftField::Message, ValidationErrorKind::Required),
                error(DraftField::Issue, ValidationErrorKind::InvalidDecimal),
            ])
        );
    }

    #[test]
    fn validated_message_returns_format_or_validation_errors() {
        assert_eq!(
            draft("fix", Some("api"), true, "handle timeout", Some(9)).validated_message(),
            Ok("fix(api)!: handle timeout (#9)".to_owned())
        );
        assert_eq!(
            draft("", None, false, "", None).validated_message(),
            Err(vec![
                error(DraftField::CommitType, ValidationErrorKind::Required),
                error(DraftField::Message, ValidationErrorKind::Required),
            ])
        );
    }
}
