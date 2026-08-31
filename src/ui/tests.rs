use ratatui::{Terminal, backend::TestBackend};
use tui_input::Input;

use super::*;
use crate::{
    commit::Footer,
    config::{Capitalization, MessagePolicy, SubjectPolicy, TerminalPunctuation, UiSections},
};

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
        footer_hint(),
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
                crate::git::StagedFile::for_display(
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
    app.focus = Focus::StagedChanges;
    app.staged_selected = 9;

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

    app.validation_error = Some(crate::commit::ValidationError {
        field: crate::commit::DraftField::Message,
        kind: crate::commit::ValidationErrorKind::Required,
    });
    let output = rendered(&app, 100, 40);
    assert!(output.contains("Error"));
    assert!(output.contains("Message is required"));
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
    app.form.message = Input::new("add footer editor".to_owned());

    let output = rendered(&app, 100, 40);
    assert!(output.contains("Subject 17/72; lowercase; no terminal punctuation"));
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
fn renders_excluded_staged_file_with_an_empty_checkbox() {
    let mut app = App::new(false);
    app.set_staged_changes(crate::git::StagedChanges {
        files: vec![
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Added,
                "src/main.rs",
                None,
            ),
            crate::git::StagedFile::for_display(
                crate::git::StagedChangeKind::Modified,
                "src/ui.rs",
                None,
            ),
        ],
        ..Default::default()
    });
    app.focus = Focus::StagedChanges;
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
    app.focus = Focus::StagedChanges;
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
        Focus::Body,
        Focus::Footers,
        Focus::Issue,
        Focus::Sign,
        Focus::StagedChanges,
        Focus::Preview,
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
fn renders_footer_editor_with_multiline_wide_unicode_content() {
    let mut app = App::new(false);
    app.form.body = Input::new("Details\nfor ��� users".to_owned());
    app.form.footers = (0..31)
        .map(|index| Footer::new(format!("Refs-{index}"), "界\nvalue".to_owned()))
        .collect();
    app.focus = Focus::Footers;
    app.handle(crate::app::AppEvent::Enter);

    let output = rendered(&app, 100, 40);
    assert!(output.contains("Footer editor"));
    assert!(output.contains("Footer name"));
    assert!(output.contains("Save footer"));
}

#[test]
fn multiline_cursor_moves_to_its_text_line_and_the_selected_footer_stays_visible() {
    let input = Input::new("first\n界界\nlast".to_owned());
    let view = super::multiline_view(&input, Rect::new(0, 0, 10, 2));
    assert_eq!(view.row, 1);
    assert_eq!(view.column, 4);

    let mut app = App::new(false);
    app.form.footers = (0..10)
        .map(|index| Footer::new(format!("Refs-{index}"), format!("#{index}")))
        .collect();
    app.focus = Focus::Footers;
    app.footer_selected = 9;
    let output = rendered(&app, 100, 40);
    assert!(output.contains("Refs-9: #9"));
}

#[test]
fn compact_footer_summary_uses_zero_when_no_footer_is_selected() {
    let mut app = App::new(false);
    app.focus = Focus::Footers;

    let output = rendered(&app, 40, 12);
    assert!(output.contains("Footers    0/0"));
}

#[test]
fn preview_viewport_clamps_scroll_and_counts_wide_wrapped_lines() {
    let viewport = super::preview_viewport("header\n界界界界\nfooter", Rect::new(0, 0, 4, 2), 99);

    assert_eq!(viewport.total_lines, 6);
    assert_eq!(viewport.max_scroll, 4);
    assert_eq!(viewport.scroll, 4);
    assert_eq!(viewport.first_line, 5);
    assert_eq!(viewport.last_line, 6);
    assert!(viewport.has_hidden_content);
}

#[test]
fn renders_an_adaptive_preview_and_the_expanded_view() {
    let mut app = App::new(false);
    app.form.message = Input::new("add endpoint".to_owned());
    app.form.body = Input::new("Details\n".repeat(20));
    app.form.footers = vec![Footer::new("Closes".to_owned(), "#42".to_owned())];
    app.focus = Focus::Preview;

    let panel = rendered(&app, 100, 60);
    assert!(panel.contains("Preview 1/"));
    assert!(panel.contains("Enter expand"));

    app.handle(crate::app::AppEvent::Enter);
    let expanded = rendered(&app, 100, 40);
    assert!(expanded.contains("Home/End start/end"));
    assert!(expanded.contains("Closes: #42"));
}

#[test]
fn clamps_scroll_values_that_exceed_the_terminal_coordinate_range() {
    assert_eq!(super::clamp_scroll(usize::MAX), u16::MAX);
}
