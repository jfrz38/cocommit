use ratatui::{Terminal, backend::TestBackend};
use tui_input::Input;

use super::*;

fn draw(app: &App, width: u16, height: u16) {
    let _ = rendered(app, width, height);
}

fn rendered(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| render(frame, app))
        .expect("rendering should not fail");
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

#[test]
fn renders_form_at_normal_terminal_size() {
    let app = App::new(false);
    draw(&app, 100, 40);
}

#[test]
fn renders_error_only_after_validation_fails() {
    let mut app = App::new(false);
    assert!(!rendered(&app, 100, 40).contains("Error"));

    app.validation_error = Some(crate::commit::ValidationError {
        field: crate::commit::DraftField::Message,
        kind: crate::commit::ValidationErrorKind::Required,
    });
    let output = rendered(&app, 100, 40);
    assert!(output.contains("Error"));
    assert!(output.contains("Message is required"));
}

#[test]
fn renders_picker_at_normal_terminal_size() {
    let mut app = App::new(false);
    app.handle(crate::app::AppEvent::Enter);
    draw(&app, 100, 40);
}

#[test]
fn renders_resize_instruction_at_small_terminal_size() {
    let app = App::new(false);
    draw(&app, 20, 10);
}

#[test]
fn renders_compact_layout_and_scrolls_to_the_focused_field() {
    let mut app = App::new(true);
    app.focus = Focus::Submit;
    draw(&app, 40, 12);
    draw(&app, 30, 8);
}

#[test]
fn renders_picker_in_a_compact_terminal() {
    let mut app = App::new(true);
    app.handle(crate::app::AppEvent::Enter);
    draw(&app, 30, 8);
}

#[test]
fn renders_help_overlay_at_both_terminal_sizes() {
    let mut app = App::new(true);
    app.handle(crate::app::AppEvent::Help);
    draw(&app, 100, 40);
    draw(&app, 30, 8);
}

#[test]
fn renders_every_focus_with_long_unicode_input_and_error() {
    let mut app = App::new(false);
    app.form.scope = Input::new("biblioteca-privada".repeat(5));
    app.form.message = Input::new("Añade soporte Unicode ".repeat(5));
    app.validation_error = Some(crate::commit::ValidationError {
        field: crate::commit::DraftField::Message,
        kind: crate::commit::ValidationErrorKind::Required,
    });

    for focus in [
        Focus::CommitType,
        Focus::Scope,
        Focus::Breaking,
        Focus::Message,
        Focus::Issue,
        Focus::Sign,
        Focus::Submit,
    ] {
        app.focus = focus;
        draw(&app, 100, 40);
    }
}

#[test]
fn renders_maximum_length_unicode_input_without_overflowing_scroll_values() {
    let mut app = App::new(false);
    app.form.message = Input::new("界".repeat(crate::app::MAX_MESSAGE_LENGTH));
    app.focus = Focus::Message;

    draw(&app, 30, 8);
    draw(&app, 100, 40);
}

#[test]
fn clamps_scroll_values_that_exceed_the_terminal_coordinate_range() {
    assert_eq!(super::clamp_scroll(usize::MAX), u16::MAX);
}
