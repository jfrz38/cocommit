use super::*;
use crate::commit::{DraftField, ValidationErrorKind};

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
        Focus::Issue,
        Focus::Sign,
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
        Focus::Sign,
        Focus::Issue,
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
fn text_editing_is_limited_to_form_text_fields() {
    let mut app = App::new(false);
    app.handle(AppEvent::Edit(Edit::Insert('x')));
    assert_eq!(app.form.commit_type.to_string(), "feat");

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
fn picker_uses_case_insensitive_substring_filtering() {
    let mut picker = TypePickerState {
        query: Input::new("EF".to_owned()),
        highlighted: 0,
    };
    assert_eq!(
        picker.choices(),
        vec![
            TypeChoice::Standard("refactor".to_owned()),
            TypeChoice::CustomQuery("EF".to_owned())
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
