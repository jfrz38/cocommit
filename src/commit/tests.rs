use crate::commit::{Capitalization, IssueStyle, MessagePolicy, TerminalPunctuation};

use super::{CommitDraft, DraftField, Footer, ValidationError, ValidationErrorKind};

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
        None,
        Vec::new(),
    )
}

fn footer(token: &str, value: &str) -> Footer {
    Footer::new(token.to_owned(), value.to_owned())
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
fn enforces_policy_and_renders_the_configured_issue_format() {
    let mut policy = MessagePolicy {
        types: vec!["fix".to_owned()],
        types_are_restricted: true,
        ..MessagePolicy::default()
    };
    policy.subject.max_length = Some(13);
    policy.subject.capitalization = Capitalization::Uppercase;
    policy.subject.terminal_punctuation = TerminalPunctuation::Require;
    policy.issue.prefix = "PROJ-".to_owned();
    policy.issue.style = IssueStyle::Plain;

    let valid = draft("fix", Some("api"), false, "Add endpoint.", Some(42));
    assert_eq!(
        valid.validated_message_with_policy(&policy),
        Ok("fix(api): Add endpoint. PROJ-42".to_owned())
    );
    assert_eq!(
        draft("feat", None, false, "Add endpoint.", None).validate_with_policy(&policy),
        Err(vec![error(
            DraftField::CommitType,
            ValidationErrorKind::NotAllowed
        )])
    );
    assert_eq!(
        draft("fix", None, false, "add endpoint", None).validate_with_policy(&policy),
        Err(vec![
            error(
                DraftField::Message,
                ValidationErrorKind::InvalidCapitalization
            ),
            error(
                DraftField::Message,
                ValidationErrorKind::TerminalPunctuationRequired
            ),
        ])
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
            None,
            Vec::new(),
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
fn renders_body_and_ordered_footers_with_exact_separation() {
    let draft = CommitDraft::new(
        "feat".to_owned(),
        Some("api".to_owned()),
        false,
        "add bulk import".to_owned(),
        None,
        Some("Import existing records.\n\nThe operation is transactional.".to_owned()),
        vec![
            footer("Closes", "#42"),
            footer("Co-authored-by", "Marta Garcia <marta@example.test>"),
        ],
    );

    assert_eq!(
        draft.validated_message(),
        Ok(
            "feat(api): add bulk import\n\nImport existing records.\n\nThe operation is transactional.\n\nCloses: #42\nCo-authored-by: Marta Garcia <marta@example.test>".to_owned()
        )
    );
}

#[test]
fn normalizes_multiline_sections_without_losing_internal_structure() {
    let draft = CommitDraft::new(
        "feat".to_owned(),
        None,
        false,
        "add import".to_owned(),
        None,
        Some("\n  Explain the behavior.\n\n  Keep this paragraph.\n".to_owned()),
        vec![footer("Refs", "\n#42\ncontinuation\n")],
    );

    assert_eq!(
        draft.render_message(),
        "feat: add import\n\nExplain the behavior.\n\n  Keep this paragraph.\n\nRefs: #42\ncontinuation"
    );

    let empty_body = CommitDraft::new(
        "feat".to_owned(),
        None,
        false,
        "add import".to_owned(),
        None,
        Some(" \n ".to_owned()),
        Vec::new(),
    );
    assert_eq!(empty_body.body, None);
}

#[test]
fn canonicalizes_both_breaking_footer_tokens_and_allows_header_marker() {
    let draft = CommitDraft::new(
        "feat".to_owned(),
        None,
        true,
        "replace configuration".to_owned(),
        None,
        None,
        vec![footer(
            "BREAKING-CHANGE",
            "old configuration is unsupported",
        )],
    );

    assert_eq!(draft.footers[0].token, "BREAKING CHANGE");
    assert_eq!(
        draft.validated_message(),
        Ok(
            "feat!: replace configuration\n\nBREAKING CHANGE: old configuration is unsupported"
                .to_owned()
        )
    );
}

#[test]
fn allows_repeatable_trailers_but_rejects_invalid_or_duplicate_breaking_footers() {
    let repeated = CommitDraft::new(
        "docs".to_owned(),
        None,
        false,
        "credit contributors".to_owned(),
        None,
        None,
        vec![
            footer("Co-authored-by", "A <a@example.test>"),
            footer("Co-authored-by", "B <b@example.test>"),
        ],
    );
    assert!(repeated.validate().is_ok());

    let invalid = CommitDraft::new(
        "feat".to_owned(),
        None,
        false,
        "change API".to_owned(),
        None,
        None,
        vec![
            Footer {
                token: "Bad Token".to_owned(),
                value: "value".to_owned(),
            },
            Footer {
                token: "Refs--more".to_owned(),
                value: "value".to_owned(),
            },
            Footer {
                token: "Refs".to_owned(),
                value: "first line\n\nsecond line".to_owned(),
            },
            Footer {
                token: "Closes".to_owned(),
                value: "   ".to_owned(),
            },
            footer("BREAKING CHANGE", "first"),
            footer("BREAKING-CHANGE", "second"),
        ],
    );

    assert_eq!(
        invalid.validate(),
        Err(vec![
            error(
                DraftField::Footer(0),
                ValidationErrorKind::ContainsForbiddenCharacter,
            ),
            error(
                DraftField::Footer(1),
                ValidationErrorKind::ContainsForbiddenCharacter,
            ),
            error(
                DraftField::Footer(2),
                ValidationErrorKind::ContainsBlankLine
            ),
            error(DraftField::Footer(3), ValidationErrorKind::Required),
            error(
                DraftField::Footer(5),
                ValidationErrorKind::DuplicateSemanticField,
            ),
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
