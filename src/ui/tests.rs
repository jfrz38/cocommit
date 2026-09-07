use super::*;
use crate::{
    app::{AppEvent, Edit, Focus},
    commit::{Capitalization, MessagePolicy, SubjectPolicy, TerminalPunctuation},
    settings::UiSections,
};
use ratatui::{Terminal, backend::TestBackend};
use tui_input::Input;

fn focus(app: &mut App, target: Focus) {
    while app.focus() != target {
        assert_eq!(app.handle(AppEvent::Tab), crate::app::AppAction::Continue);
    }
}

fn enter_text(app: &mut App, target: Focus, value: impl Into<String>) {
    focus(app, target);
    app.handle(AppEvent::Paste(value.into()));
}

fn add_footer(app: &mut App, token: &str, value: &str) {
    focus(app, Focus::Footers);
    app.handle(AppEvent::Edit(Edit::Insert('a')));
    app.handle(AppEvent::Paste(token.to_owned()));
    app.handle(AppEvent::Enter);
    app.handle(AppEvent::Paste(value.to_owned()));
    app.handle(AppEvent::Submit);
}

fn draw(app: &App, width: u16, height: u16) {
    let _ = rendered(app, width, height);
}

fn rendered(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| {
            let _ = render(frame, app);
        })
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
fn footer_hint_always_describes_space_as_toggle() {
    assert_eq!(
        components::footer_hint(),
        "F1 Help  |  Up/Down Navigate  |  Space Toggle  |  Ctrl+Enter Commit  |  Esc Cancel"
    );
}

#[test]
fn hidden_sections_consume_no_space_or_expose_actions() {
    let mut app = App::new(false).with_sections(UiSections {
        staged_changes: false,
        body: false,
        footers: false,
        issue: false,
    });
    app.handle(crate::app::AppEvent::Help);

    for (width, height) in [(100, 40), (40, 12)] {
        let output = rendered(&app, width, height);
        for hidden_label in [
            "Body",
            "Footers",
            "Issue",
            "Staged",
            "Add footer",
            "BREAKING CHANGE",
        ] {
            assert!(
                !output.contains(hidden_label),
                "{hidden_label} should be hidden at {width}x{height}"
            );
        }
    }
}

#[test]
fn all_hidden_sections_use_compact_layout_at_expanded_cutoff() {
    let mut app = App::new(false).with_sections(UiSections {
        staged_changes: false,
        body: false,
        footers: false,
        issue: false,
    });
    focus(&mut app, Focus::Submit);

    let output = rendered(&app, 50, 24);

    assert!(output.contains("Sign commit [ ]"));
    assert!(output.contains("Preview    scroll 0"));
    assert!(output.contains("[ Commit ]"));
}

#[test]
fn each_hidden_section_is_absent_in_both_layouts() {
    for (sections, hidden_label) in [
        (
            UiSections {
                staged_changes: false,
                ..UiSections::default()
            },
            "Staged",
        ),
        (
            UiSections {
                body: false,
                ..UiSections::default()
            },
            "Body",
        ),
        (
            UiSections {
                footers: false,
                ..UiSections::default()
            },
            "Footers",
        ),
        (
            UiSections {
                issue: false,
                ..UiSections::default()
            },
            "Issue",
        ),
    ] {
        let app = App::new(false).with_sections(sections);
        for (width, height) in [(100, 40), (40, 12)] {
            assert!(
                !rendered(&app, width, height).contains(hidden_label),
                "{hidden_label} should be hidden at {width}x{height}"
            );
        }
    }
}

#[test]
fn renders_every_section_combination_in_expanded_and_compact_layouts() {
    for flags in 0..16 {
        let app = App::new(false).with_sections(UiSections {
            staged_changes: flags & 1 != 0,
            body: flags & 2 != 0,
            footers: flags & 4 != 0,
            issue: flags & 8 != 0,
        });
        draw(&app, 100, 40);
        draw(&app, 40, 12);
    }
}

#[test]
fn renders_staged_change_counts_and_a_scrolled_file_list() {
    let mut app = App::new(false);
    app.set_staged_changes(crate::git::StagedChanges {
        files: (0..10)
            .map(|index| {
                crate::git::StagedFile::from_display_paths(
                    crate::git::StagedChangeKind::Added,
                    format!("src/archivo-{index}.rs"),
                    None,
                )
            })
            .collect(),
        insertions: 12,
        deletions: 3,
        binary_files: 1,
    });
    focus(&mut app, Focus::StagedChanges);
    for _ in 0..9 {
        app.handle(AppEvent::Down);
    }

    let output = rendered(&app, 100, 40);
    assert!(output.contains("Staged"));
    assert!(output.contains("A:10 M:0 D:0 R:0"));
    assert!(output.contains("> [x] A src/archivo-9.rs"));
    assert!(output.contains("archivo-9.rs"));
}

#[test]
fn renders_error_only_after_validation_fails() {
    let mut app = App::new(false);
    assert!(!rendered(&app, 100, 40).contains("Error"));

    app.handle(AppEvent::Submit);
    let output = rendered(&app, 100, 40);
    assert!(output.contains("Error"));
    assert!(output.contains("Message is required"));
}

#[test]
fn renders_input_feedback_before_validation_feedback() {
    let mut app = App::new(false);
    app.handle(AppEvent::Submit);
    enter_text(
        &mut app,
        Focus::Message,
        "a".repeat(crate::app::MAX_MESSAGE_LENGTH),
    );
    app.handle(AppEvent::Edit(Edit::Insert('a')));

    let output = rendered(&app, 100, 40);
    assert!(output.contains("Field is too long"));
    assert!(!output.contains("Message is required"));
}

#[test]
fn renders_subject_indicator_for_the_active_policy() {
    let policy = MessagePolicy {
        subject: SubjectPolicy {
            max_length: Some(72),
            capitalization: Capitalization::Lowercase,
            terminal_punctuation: TerminalPunctuation::Forbid,
        },
        ..MessagePolicy::default()
    };
    let mut app = App::new(false).with_message_policy(&policy);
    enter_text(&mut app, Focus::Message, "add footer editor");

    let output = rendered(&app, 100, 40);
    assert!(output.contains("Subject 17/72; lowercase; no terminal punctuation"));
}

#[test]
fn presentation_helpers_preserve_form_labels_and_picker_titles() {
    assert_eq!(components::scope_label(false), "Scope");
    assert_eq!(components::scope_label(true), "Scope (Enter suggestions)");
    assert_eq!(components::type_picker_title(false), " Select commit type ");
    assert_eq!(
        components::type_picker_title(true),
        " Allowed commit types "
    );
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
    focus(&mut app, Focus::Submit);
    draw(&app, 40, 12);
    draw(&app, 30, 8);
}

#[test]
fn renders_excluded_staged_file_with_an_empty_checkbox() {
    let mut app = App::new(false);
    app.set_staged_changes(crate::git::StagedChanges {
        files: vec![
            crate::git::StagedFile::from_display_paths(
                crate::git::StagedChangeKind::Added,
                "src/main.rs",
                None,
            ),
            crate::git::StagedFile::from_display_paths(
                crate::git::StagedChangeKind::Modified,
                "src/ui.rs",
                None,
            ),
        ],
        ..Default::default()
    });
    focus(&mut app, Focus::StagedChanges);
    app.handle(crate::app::AppEvent::Space);

    let output = rendered(&app, 100, 40);
    assert!(output.contains("1/2 included"));
    assert!(output.contains("> [ ] A src/main.rs"));
}

#[test]
fn keeps_preview_below_commit_in_a_tall_compact_terminal() {
    let app = App::new(false);
    let width = 40usize;
    let output = rendered(&app, width as u16, 40);
    let commit = output.find("[ Commit ]").expect("Commit should render") / width;
    let preview = output.find("Preview 1/").expect("preview should render") / width;

    assert_eq!(preview, commit + 1);
}

#[test]
fn renders_staged_detail_at_the_compact_layout_boundary() {
    let mut app = App::new(false);
    focus(&mut app, Focus::StagedChanges);
    let output = rendered(&app, 50, 30);

    assert!(output.contains("Staged"));
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
    enter_text(&mut app, Focus::Scope, "biblioteca-privada".repeat(5));
    enter_text(&mut app, Focus::Message, "Añade soporte Unicode ".repeat(5));
    app.handle(AppEvent::Submit);

    for target in [
        Focus::CommitType,
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
    ] {
        focus(&mut app, target);
        draw(&app, 100, 40);
    }
}

#[test]
fn renders_maximum_length_unicode_input_without_overflowing_scroll_values() {
    let mut app = App::new(false);
    enter_text(
        &mut app,
        Focus::Message,
        "界".repeat(crate::app::MAX_MESSAGE_LENGTH),
    );

    draw(&app, 30, 8);
    draw(&app, 100, 40);
}

#[test]
fn renders_footer_editor_with_multiline_wide_unicode_content() {
    let mut app = App::new(false);
    enter_text(&mut app, Focus::Body, "Details\nfor ��� users");
    for index in 0..31 {
        add_footer(&mut app, &format!("Refs-{index}"), "界\nvalue");
    }
    focus(&mut app, Focus::Footers);
    app.handle(crate::app::AppEvent::Enter);

    let output = rendered(&app, 100, 40);
    assert!(output.contains("Footer editor"));
    assert!(output.contains("Footer name"));
    assert!(output.contains("Save footer"));
}

#[test]
fn multiline_cursor_moves_to_its_text_line_and_the_selected_footer_stays_visible() {
    let input = Input::new("first\n界界\nlast".to_owned());
    let view = components::multiline_view(&input, Rect::new(0, 0, 10, 2));
    assert_eq!(view.row, 1);
    assert_eq!(view.column, 4);

    let mut app = App::new(false);
    for index in 0..10 {
        add_footer(&mut app, &format!("Refs-{index}"), &format!("#{index}"));
    }
    focus(&mut app, Focus::Footers);
    let output = rendered(&app, 100, 40);
    assert!(output.contains("Refs-9: #9"));
}

#[test]
fn compact_footer_summary_uses_zero_when_no_footer_is_selected() {
    let mut app = App::new(false);
    focus(&mut app, Focus::Footers);

    let output = rendered(&app, 40, 12);
    assert!(output.contains("Footers    0/0"));
}

#[test]
fn renders_an_adaptive_preview_and_the_expanded_view() {
    let mut app = App::new(false);
    enter_text(&mut app, Focus::Message, "add endpoint");
    enter_text(&mut app, Focus::Body, "Details\n".repeat(20));
    add_footer(&mut app, "Closes", "#42");
    focus(&mut app, Focus::Preview);

    let panel = rendered(&app, 100, 60);
    assert!(panel.contains("Preview 1/"));
    assert!(panel.contains("Enter expand"));

    app.handle(crate::app::AppEvent::Enter);
    let expanded = rendered(&app, 100, 40);
    assert!(expanded.contains("Home/End start/end"));
    assert!(expanded.contains("Closes: #42"));
}
