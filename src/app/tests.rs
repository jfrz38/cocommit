use super::*;
use crate::commit::{DraftField, ValidationErrorKind};
use crate::config::UiSections;

fn focus(app: &mut App, target: Focus) {
    while app.focus != target {
        assert_eq!(app.handle(AppEvent::Tab), AppAction::Continue);
    }
}

#[test]
fn tab_cycles_through_all_focus_targets() {
    let mut app = App::new(false);
    let expected = [
        Focus::Scope,
        Focus::Breaking,
        Focus::Message,
        Focus::Body,
        Focus::Footers,
        Focus::Issue,
        Focus::Sign,
        Focus::StagedChanges,
        Focus::Preview,
        Focus::Submit,
        Focus::CommitType,
    ];

    for target in expected {
        app.handle(AppEvent::Tab);
        assert_eq!(app.focus, target);
    }
}

#[test]
fn back_tab_cycles_in_reverse() {
    let mut app = App::new(false);
    let expected = [
        Focus::Submit,
        Focus::Preview,
        Focus::StagedChanges,
        Focus::Sign,
        Focus::Issue,
        Focus::Footers,
        Focus::Body,
        Focus::Message,
        Focus::Breaking,
        Focus::Scope,
        Focus::CommitType,
    ];

    for target in expected {
        app.handle(AppEvent::BackTab);
        assert_eq!(app.focus, target);
    }
}

#[test]
fn every_section_combination_has_a_complete_focus_path() {
    for flags in 0..16 {
        let sections = UiSections {
            staged_changes: flags & 1 != 0,
            body: flags & 2 != 0,
            footers: flags & 4 != 0,
            issue: flags & 8 != 0,
        };
        let mut app = App::new(false).with_sections(sections);
        let expected = app.visible_focuses();

        for target in expected
            .iter()
            .copied()
            .cycle()
            .skip(1)
            .take(expected.len())
        {
            app.handle(AppEvent::Tab);
            assert_eq!(app.focus, target, "failed configuration {flags:04b}");
        }
        assert_eq!(
            app.focus,
            Focus::CommitType,
            "failed configuration {flags:04b}"
        );
    }
}

#[test]
fn hidden_optional_sections_are_absent_from_draft_and_interaction() {
    let sections = UiSections {
        staged_changes: true,
        body: false,
        footers: false,
        issue: false,
    };
    let mut app = App::new(false).with_sections(sections);
    app.form.body = Input::new("must not render".to_owned());
    app.form.issue = Input::new("42".to_owned());
    app.form.footers = vec![Footer::new("Closes".to_owned(), "#42".to_owned())];
    app = app.with_sections(sections);

    assert_eq!(
        app.visible_focuses(),
        vec![
            Focus::CommitType,
            Focus::Scope,
            Focus::Breaking,
            Focus::Message,
            Focus::Sign,
            Focus::StagedChanges,
            Focus::Preview,
            Focus::Submit,
        ]
    );
    assert_eq!(app.preview(), "feat: ");
    assert!(app.form.body.to_string().is_empty());
    assert!(app.form.issue.to_string().is_empty());
    assert!(app.form.footers.is_empty());
}

#[test]
fn hidden_staged_changes_keep_all_files_committable_without_exclusions() {
    let mut app = App::new(false).with_sections(UiSections {
        staged_changes: false,
        ..UiSections::default()
    });
    app.set_staged_changes(crate::git::StagedChanges {
        files: vec![
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Added,
                "one.txt",
                None,
            ),
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Modified,
                "two.txt",
                None,
            ),
        ],
        ..Default::default()
    });

    assert!(!app.visible_focuses().contains(&Focus::StagedChanges));
    assert_eq!(app.included_staged_count(), 2);
    assert!(app.excluded_staged_files().is_empty());
    assert!(!app.staged_file_is_included(0));
}

#[test]
fn staged_changes_select_files_and_leave_the_list_at_its_boundaries() {
    let mut app = App::new(false);
    app.set_staged_changes(crate::git::StagedChanges {
        files: vec![
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Added,
                "one.txt",
                None,
            ),
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Modified,
                "two.txt",
                None,
            ),
        ],
        ..Default::default()
    });
    focus(&mut app, Focus::StagedChanges);

    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::StagedChanges);
    assert_eq!(app.staged_selected, 1);
    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::Preview);
    app.handle(AppEvent::Tab);
    assert_eq!(app.focus, Focus::Submit);
    app.handle(AppEvent::BackTab);
    assert_eq!(app.focus, Focus::Preview);
    app.handle(AppEvent::BackTab);
    assert_eq!(app.focus, Focus::StagedChanges);
    app.handle(AppEvent::Edit(Edit::Home));
    assert_eq!(app.staged_selected, 0);
    app.handle(AppEvent::Up);
    assert_eq!(app.focus, Focus::Sign);
    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::StagedChanges);
    app.handle(AppEvent::Edit(Edit::Home));
    assert_eq!(app.staged_selected, 0);
    app.handle(AppEvent::Edit(Edit::End));
    assert_eq!(app.staged_selected, 1);
}

#[test]
fn space_only_toggles_boolean_controls() {
    let mut app = App::new(false);
    app.handle(AppEvent::Space);
    assert!(!app.form.breaking);
    assert!(!app.sign);

    focus(&mut app, Focus::Breaking);
    app.handle(AppEvent::Space);
    assert!(app.form.breaking);

    focus(&mut app, Focus::Sign);
    app.handle(AppEvent::Space);
    assert!(app.sign);
}

#[test]
fn space_on_an_empty_staged_list_is_a_noop() {
    let mut app = App::new(false);
    focus(&mut app, Focus::StagedChanges);

    assert_eq!(app.handle(AppEvent::Space), AppAction::Continue);
    assert!(app.excluded_staged_files().is_empty());
}

#[test]
fn space_excludes_the_selected_file_but_keeps_one_file_included() {
    let mut app = App::new(false);
    app.set_staged_changes(crate::git::StagedChanges {
        files: vec![
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Added,
                "one.txt",
                None,
            ),
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Modified,
                "two.txt",
                None,
            ),
        ],
        ..Default::default()
    });
    focus(&mut app, Focus::StagedChanges);
    app.handle(AppEvent::Down);

    assert_eq!(app.handle(AppEvent::Space), AppAction::Continue);
    assert_eq!(app.included_staged_count(), 1);
    assert_eq!(
        app.excluded_staged_files(),
        vec![crate::git::StagedFile::for_display(
            crate::git::StagedChangeKind::Modified,
            "two.txt",
            None,
        )]
    );

    app.handle(AppEvent::Up);
    assert_eq!(app.handle(AppEvent::Space), AppAction::Continue);
    assert_eq!(
        app.status_message(),
        Some("Cannot unstage the last staged file")
    );
    assert!(app.status_is_error());
}

#[test]
fn typing_on_type_opens_the_picker_with_the_typed_query() {
    let mut app = App::new(false);
    app.handle(AppEvent::Edit(Edit::Insert('d')));
    let Mode::TypePicker(picker) = &app.mode else {
        panic!("type picker should be open");
    };
    assert_eq!(picker.query.to_string(), "d");
    assert_eq!(picker.choices()[0], TypeChoice::Standard("docs".to_owned()));

    app.handle(AppEvent::Enter);
    assert_eq!(app.form.commit_type.to_string(), "docs");

    focus(&mut app, Focus::Scope);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    assert_eq!(app.form.scope.to_string(), "a");

    focus(&mut app, Focus::Message);
    app.handle(AppEvent::Edit(Edit::Insert('b')));
    app.handle(AppEvent::Space);
    app.handle(AppEvent::Edit(Edit::Insert('c')));
    assert_eq!(app.form.message.to_string(), "b c");
}

#[test]
fn enter_opens_picker_and_selects_standard_type() {
    let mut app = App::new(false);
    app.handle(AppEvent::Enter);
    assert!(matches!(app.mode, Mode::TypePicker(_)));

    app.handle(AppEvent::Edit(Edit::Insert('f')));
    app.handle(AppEvent::Edit(Edit::Insert('i')));
    app.handle(AppEvent::Edit(Edit::Insert('x')));
    assert_eq!(app.handle(AppEvent::Enter), AppAction::Continue);
    assert!(matches!(app.mode, Mode::Form));
    assert_eq!(app.form.commit_type.to_string(), "fix");
    assert_eq!(app.focus, Focus::Scope);
}

#[test]
fn picker_uses_case_insensitive_prefix_filtering() {
    let mut picker = TypePickerState {
        query: Input::new("DO".to_owned()),
        highlighted: 0,
    };
    assert_eq!(
        picker.choices(),
        vec![
            TypeChoice::Standard("docs".to_owned()),
            TypeChoice::CustomQuery("DO".to_owned())
        ]
    );

    picker.query = Input::new("feat".to_owned());
    assert_eq!(
        picker.choices(),
        vec![TypeChoice::Standard("feat".to_owned())]
    );
}

#[test]
fn picker_custom_query_is_selected() {
    let mut app = App::new(false);
    app.handle(AppEvent::Enter);
    for character in "deploy".chars() {
        app.handle(AppEvent::Edit(Edit::Insert(character)));
    }
    app.handle(AppEvent::Enter);

    assert_eq!(app.form.commit_type.to_string(), "deploy");
    assert_eq!(app.focus, Focus::Scope);
}

#[test]
fn picker_accepts_a_query_without_a_matching_prefix() {
    let mut app = App::new(false);
    app.handle(AppEvent::Edit(Edit::Insert('d')));
    app.handle(AppEvent::Edit(Edit::Insert('o')));
    app.handle(AppEvent::Edit(Edit::Insert('s')));
    app.handle(AppEvent::Enter);

    assert_eq!(app.form.commit_type.to_string(), "dos");
    assert_eq!(app.focus, Focus::Scope);
}

#[test]
fn picker_escape_keeps_existing_type_and_tabs_do_nothing() {
    let mut app = App::new(false);
    app.handle(AppEvent::Enter);
    app.handle(AppEvent::Tab);
    app.handle(AppEvent::BackTab);
    app.handle(AppEvent::Escape);

    assert!(matches!(app.mode, Mode::Form));
    assert_eq!(app.form.commit_type.to_string(), "feat");
    assert_eq!(app.focus, Focus::CommitType);
}

#[test]
fn cancelling_a_type_search_keeps_existing_validation_feedback() {
    let mut app = App::new(false);
    app.validation_error = Some(ValidationError {
        field: DraftField::Message,
        kind: ValidationErrorKind::Required,
    });

    app.handle(AppEvent::Edit(Edit::Insert('d')));
    app.handle(AppEvent::Escape);

    assert_eq!(
        app.validation_error,
        Some(ValidationError {
            field: DraftField::Message,
            kind: ValidationErrorKind::Required,
        })
    );
}

#[test]
fn picker_navigation_wraps() {
    let mut app = App::new(false);
    app.handle(AppEvent::Enter);
    app.handle(AppEvent::Up);
    let Mode::TypePicker(picker) = &app.mode else {
        panic!("type picker should be open");
    };
    assert_eq!(picker.highlighted, picker.choices().len() - 1);
}

#[test]
fn paste_sanitizes_line_breaks_in_form_and_picker() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Message);
    app.handle(AppEvent::Paste("add\r\nnew\nfeature".to_owned()));
    assert_eq!(app.form.message.to_string(), "add new feature");

    focus(&mut app, Focus::CommitType);
    app.handle(AppEvent::Enter);
    app.handle(AppEvent::Paste("custom\r\ntype".to_owned()));
    let Mode::TypePicker(picker) = &app.mode else {
        panic!("type picker should be open");
    };
    assert_eq!(picker.query.to_string(), "custom type");
}

#[test]
fn input_rejects_control_characters_without_changing_the_field() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Message);

    app.handle(AppEvent::Edit(Edit::Insert('\u{1b}')));

    assert!(app.form.message.to_string().is_empty());
    assert_eq!(
        app.validation_message(),
        Some("Control characters are not supported")
    );
    app.handle(AppEvent::Paste("valid\0paste".to_owned()));
    assert!(app.form.message.to_string().is_empty());
    assert_eq!(
        app.validation_message(),
        Some("Paste contains unsupported control characters or is too large")
    );

    focus(&mut app, Focus::Footers);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    app.handle(AppEvent::Enter);
    {
        let Mode::FooterEditor(editor) = &mut app.mode else {
            panic!("footer editor should open");
        };
        editor.focus = FooterEditorFocus::Value;
        editor.value = Input::new("a".repeat(MAX_FOOTER_VALUE_LENGTH - 1));
    }
    app.handle(AppEvent::Paste("bc".to_owned()));
    let Mode::FooterEditor(editor) = &app.mode else {
        panic!("footer editor should remain open");
    };
    assert_eq!(
        editor.value.to_string().chars().count(),
        MAX_FOOTER_VALUE_LENGTH - 1
    );
    assert_eq!(
        app.validation_message(),
        Some("Paste exceeds the field limit")
    );
}

#[test]
fn validation_feedback_takes_priority_over_operation_feedback() {
    let mut app = App::new(false);
    app.validation_error = Some(ValidationError {
        field: DraftField::Message,
        kind: ValidationErrorKind::Required,
    });
    app.set_operation_error("Cannot unstage the last staged file");

    assert_eq!(app.status_message(), Some("Message is required"));
    assert!(app.status_is_error());
}

#[test]
fn picker_clears_rejected_input_feedback_when_cancelled() {
    let mut app = App::new(false);
    app.validation_error = Some(ValidationError {
        field: DraftField::Message,
        kind: ValidationErrorKind::Required,
    });
    focus(&mut app, Focus::CommitType);
    app.handle(AppEvent::Edit(Edit::Insert('\u{1b}')));
    assert_eq!(
        app.validation_message(),
        Some("Control characters are not supported")
    );

    app.handle(AppEvent::Escape);

    assert_eq!(app.validation_message(), Some("Message is required"));
}

#[test]
fn input_enforces_field_and_paste_limits_without_partial_insertion() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Issue);
    app.handle(AppEvent::Paste("9".repeat(MAX_ISSUE_LENGTH)));
    assert_eq!(app.form.issue.to_string(), "9".repeat(MAX_ISSUE_LENGTH));

    app.handle(AppEvent::Edit(Edit::Insert('9')));
    assert_eq!(app.form.issue.to_string(), "9".repeat(MAX_ISSUE_LENGTH));
    assert_eq!(app.validation_message(), Some("Field is too long"));

    focus(&mut app, Focus::Message);
    app.handle(AppEvent::Paste("a".repeat(MAX_MESSAGE_LENGTH + 1)));
    assert!(app.form.message.to_string().is_empty());
    assert_eq!(
        app.validation_message(),
        Some("Paste exceeds the field limit")
    );

    app.handle(AppEvent::Paste("b".repeat(MAX_PASTE_LENGTH + 1)));
    assert!(app.form.message.to_string().is_empty());
    assert_eq!(
        app.validation_message(),
        Some("Paste contains unsupported control characters or is too large")
    );
}

#[test]
fn unicode_input_counts_characters_instead_of_bytes() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Scope);
    app.handle(AppEvent::Paste("界".repeat(MAX_SCOPE_LENGTH)));

    assert_eq!(app.form.scope.to_string().chars().count(), MAX_SCOPE_LENGTH);
    app.handle(AppEvent::Edit(Edit::Insert('界')));
    assert_eq!(app.form.scope.to_string().chars().count(), MAX_SCOPE_LENGTH);
    assert_eq!(app.validation_message(), Some("Field is too long"));
}

#[test]
fn cancel_actions_cancel_from_both_modes() {
    let mut app = App::new(false);
    assert_eq!(app.handle(AppEvent::Escape), AppAction::Cancel);
    app.handle(AppEvent::Enter);
    assert_eq!(app.handle(AppEvent::Cancel), AppAction::Cancel);
}

#[test]
fn invalid_submission_focuses_the_first_invalid_field() {
    let mut app = App::new(false);
    app.form.commit_type = Input::default();
    focus(&mut app, Focus::Submit);

    assert_eq!(app.handle(AppEvent::Enter), AppAction::Continue);
    assert_eq!(app.focus, Focus::CommitType);
    assert_eq!(
        app.validation_error,
        Some(ValidationError {
            field: DraftField::CommitType,
            kind: ValidationErrorKind::Required
        })
    );
}

#[test]
fn valid_submission_preserves_signing_choice_and_clears_errors_on_edit() {
    let mut app = App::new(true);
    focus(&mut app, Focus::Submit);
    assert_eq!(app.handle(AppEvent::Enter), AppAction::Continue);
    assert!(app.validation_error.is_some());

    focus(&mut app, Focus::Message);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    assert!(app.validation_error.is_none());
    focus(&mut app, Focus::Submit);
    assert_eq!(app.handle(AppEvent::Enter), AppAction::Submit);
    assert!(app.sign);
}

#[test]
fn arrow_keys_cycle_through_form_focus_targets() {
    let mut app = App::new(false);
    assert_eq!(app.handle(AppEvent::Down), AppAction::Continue);
    assert_eq!(app.focus, Focus::Scope);
    assert_eq!(app.handle(AppEvent::Up), AppAction::Continue);
    assert_eq!(app.focus, Focus::CommitType);
    assert_eq!(app.handle(AppEvent::Up), AppAction::Continue);
    assert_eq!(app.focus, Focus::Submit);
}

#[test]
fn help_toggles_without_losing_the_previous_mode() {
    let mut app = App::new(false);
    app.focus = Focus::Message;
    app.handle(AppEvent::Help);
    assert!(matches!(app.mode, Mode::Help(_)));
    app.handle(AppEvent::Escape);
    assert!(matches!(app.mode, Mode::Form));
    assert_eq!(app.focus, Focus::Message);

    app.focus = Focus::CommitType;
    app.handle(AppEvent::Enter);
    app.handle(AppEvent::Help);
    app.handle(AppEvent::Help);
    assert!(matches!(app.mode, Mode::TypePicker(_)));
}

#[test]
fn submit_event_submits_from_any_form_field_but_not_the_picker() {
    let mut app = App::new(true);
    app.form.message = Input::new("add endpoint".to_owned());
    assert_eq!(app.handle(AppEvent::Submit), AppAction::Submit);

    app.handle(AppEvent::Enter);
    assert!(matches!(app.mode, Mode::TypePicker(_)));
    assert_eq!(app.handle(AppEvent::Submit), AppAction::Continue);
    assert!(matches!(app.mode, Mode::TypePicker(_)));
}

#[test]
fn preview_uses_the_canonical_formatter_for_incomplete_forms() {
    let mut app = App::new(false);
    assert_eq!(app.preview(), "feat: ");

    app.form.scope = Input::new("api".to_owned());
    app.form.breaking = true;
    app.form.message = Input::new("add endpoint".to_owned());
    app.form.issue = Input::new("42".to_owned());
    assert_eq!(app.preview(), "feat(api)!: add endpoint (#42)");
}

#[test]
fn preview_omits_an_invalid_issue_until_submission() {
    let mut app = App::new(false);
    app.form.message = Input::new("add endpoint".to_owned());
    app.form.issue = Input::new("not-a-number".to_owned());

    assert_eq!(app.preview(), "feat: add endpoint");
}

#[test]
fn preview_can_expand_scroll_and_restore_the_form() {
    let mut app = App::new(false);
    app.set_preview_scroll_limits(3, Some(8));
    app.form.message = Input::new("add endpoint".to_owned());
    app.form.body = Input::new("one\ntwo\nthree".to_owned());
    focus(&mut app, Focus::Preview);

    app.handle(AppEvent::Enter);
    assert!(matches!(app.mode, Mode::PreviewExpanded));
    app.handle(AppEvent::Down);
    assert_eq!(app.preview_scroll, 1);
    app.handle(AppEvent::Edit(Edit::End));
    assert_eq!(app.preview_scroll, 8);
    app.handle(AppEvent::Edit(Edit::Home));
    assert_eq!(app.preview_scroll, 0);
    app.handle(AppEvent::Help);
    app.handle(AppEvent::Escape);
    assert!(matches!(app.mode, Mode::PreviewExpanded));
    app.handle(AppEvent::Escape);
    assert!(matches!(app.mode, Mode::Form));
    assert_eq!(app.focus, Focus::Preview);
}

#[test]
fn preview_scrolls_before_arrow_keys_leave_its_boundaries() {
    let mut app = App::new(false);
    app.set_preview_scroll_limits(2, None);
    focus(&mut app, Focus::Preview);

    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::Preview);
    assert_eq!(app.preview_scroll, 1);
    app.handle(AppEvent::Down);
    assert_eq!(app.preview_scroll, 2);
    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::Submit);

    app.handle(AppEvent::Up);
    assert_eq!(app.focus, Focus::Preview);
    app.handle(AppEvent::Up);
    assert_eq!(app.preview_scroll, 1);
    app.handle(AppEvent::Up);
    assert_eq!(app.preview_scroll, 0);
    app.handle(AppEvent::Up);
    assert_eq!(app.focus, Focus::StagedChanges);
}

#[test]
fn preview_scroll_resets_when_the_draft_changes() {
    let mut app = App::new(false);
    app.preview_scroll = 8;
    focus(&mut app, Focus::Message);

    app.handle(AppEvent::Edit(Edit::Insert('a')));

    assert_eq!(app.preview_scroll, 0);
}

#[test]
fn body_and_footer_editor_build_the_canonical_preview() {
    let mut app = App::new(false);
    app.form.message = Input::new("add endpoint".to_owned());
    focus(&mut app, Focus::Body);
    app.handle(AppEvent::Paste(
        "Explain the endpoint\n\nWith examples".to_owned(),
    ));
    focus(&mut app, Focus::Footers);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    app.handle(AppEvent::Edit(Edit::Insert('c')));
    app.handle(AppEvent::Enter);
    let Mode::FooterEditor(editor) = &mut app.mode else {
        panic!("footer editor should open");
    };
    editor.token = Input::new("Closes".to_owned());
    editor.value = Input::new("#42".to_owned());
    app.handle(AppEvent::Submit);
    assert_eq!(
        app.preview(),
        "feat: add endpoint\n\nExplain the endpoint\n\nWith examples\n\nCloses: #42"
    );
}

#[test]
fn footer_editor_can_add_breaking_delete_and_reorder() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Footers);
    app.handle(AppEvent::Space);
    let Mode::FooterEditor(editor) = &mut app.mode else {
        panic!("footer editor should open");
    };
    editor.value = Input::new("API changed".to_owned());
    app.handle(AppEvent::Submit);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    app.handle(AppEvent::Edit(Edit::Insert('r')));
    app.handle(AppEvent::Enter);
    let Mode::FooterEditor(editor) = &mut app.mode else {
        panic!("footer editor should open after choosing a name");
    };
    editor.value = Input::new("#9".to_owned());
    app.handle(AppEvent::Submit);
    app.handle(AppEvent::MoveFooter(-1));
    assert_eq!(app.form.footers[0].token, "Refs");
    app.handle(AppEvent::Edit(Edit::Delete));
    assert_eq!(app.form.footers.len(), 1);

    app.handle(AppEvent::Enter);
    let Mode::FooterEditor(editor) = &mut app.mode else {
        panic!("selected footer should open for editing");
    };
    editor.value = Input::new("Updated API change".to_owned());
    app.handle(AppEvent::Submit);
    assert_eq!(app.form.footers[0].value, "Updated API change");
}

#[test]
fn footer_name_picker_supports_standard_custom_and_cancelled_additions() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Footers);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    assert!(matches!(app.mode, Mode::FooterNamePicker(_)));
    app.handle(AppEvent::Edit(Edit::Insert('x')));
    app.handle(AppEvent::Enter);
    let Mode::FooterEditor(editor) = &app.mode else {
        panic!("custom footer editor should open");
    };
    assert_eq!(editor.token.to_string(), "x");
    assert_eq!(editor.focus, FooterEditorFocus::Value);
    app.handle(AppEvent::Escape);
    assert!(matches!(app.mode, Mode::Form));
    assert!(app.form.footers.is_empty());

    app.handle(AppEvent::Edit(Edit::Insert('a')));
    app.handle(AppEvent::Down);
    app.handle(AppEvent::Enter);
    let Mode::FooterEditor(editor) = &app.mode else {
        panic!("standard footer editor should open");
    };
    assert_eq!(editor.token.to_string(), "Closes");
}

#[test]
fn footer_editor_focus_wraps_in_both_directions() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Footers);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    app.handle(AppEvent::Enter);
    app.handle(AppEvent::BackTab);
    app.handle(AppEvent::BackTab);
    let Mode::FooterEditor(editor) = &app.mode else {
        panic!("footer editor should remain open");
    };
    assert_eq!(editor.focus, FooterEditorFocus::Save);
}

#[test]
fn footer_navigation_leaves_at_boundaries_and_can_remove_every_footer() {
    let mut app = App::new(false);
    app.form.footers = vec![
        Footer::new("Refs".to_owned(), "#1".to_owned()),
        Footer::new("Closes".to_owned(), "#2".to_owned()),
    ];
    focus(&mut app, Focus::Footers);

    app.handle(AppEvent::Down);
    assert_eq!(app.footer_selected, 1);
    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::Issue);
    app.handle(AppEvent::Up);
    assert_eq!(app.focus, Focus::Footers);
    app.handle(AppEvent::Edit(Edit::Delete));
    app.handle(AppEvent::Edit(Edit::Delete));
    assert!(app.form.footers.is_empty());
    app.handle(AppEvent::Down);
    assert_eq!(app.focus, Focus::Issue);
}
