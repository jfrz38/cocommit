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
