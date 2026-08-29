use ratatui::{Terminal, backend::TestBackend};
use tui_input::Input;

use super::*;

fn draw(app: &App, width: u16, height: u16) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| render(frame, app))
        .expect("rendering should not fail");
}

#[test]
fn renders_form_at_normal_terminal_size() {
    let app = App::new(false);
    draw(&app, 100, 40);
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
fn renders_every_focus_with_long_unicode_input_and_status() {
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
