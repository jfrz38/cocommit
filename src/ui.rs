//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use tui_input::Input;
use unicode_width::UnicodeWidthChar;

use crate::app::{
    App, Focus, FooterEditorFocus, FooterEditorState, FooterNamePickerState, Mode, TypeChoice,
    TypePickerState,
};

const MIN_WIDTH: u16 = 30;
const MIN_HEIGHT: u16 = 8;
const EXPANDED_WIDTH: u16 = 50;
const EXPANDED_HEIGHT: u16 = 39;
const EXPANDED_HEIGHT_WITH_STATUS: u16 = 42;
const COMPACT_FORM_ROWS: u16 = 11;
const FOOTER: &str = "F1 Help  Up/Down Navigate  A Add footer  Ctrl+Enter Commit  Esc Cancel";

#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewScrollLimits {
    pub form: u16,
    pub expanded: Option<u16>,
}

/// Draws the complete application state.
pub fn render(frame: &mut Frame, app: &App) -> PreviewScrollLimits {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_resize_instruction(frame, area);
        return PreviewScrollLimits::default();
    }

    let expanded_height = if app.status_message().is_some() {
        EXPANDED_HEIGHT_WITH_STATUS
    } else {
        EXPANDED_HEIGHT
    };
    if area.width >= EXPANDED_WIDTH && area.height >= expanded_height {
        render_expanded(frame, app, area)
    } else {
        render_compact(frame, app, area)
    }
}

fn render_expanded(frame: &mut Frame, app: &App, area: Rect) -> PreviewScrollLimits {
    let outer = Block::default().borders(Borders::ALL).title(" cocommit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let staged_height = inner
        .height
        .saturating_sub(33 + 3 * u16::from(app.status_message().is_some()))
        .clamp(4, 9);
    let preview_height = inner
        .height
        .saturating_sub(30 + staged_height + 3 * u16::from(app.status_message().is_some()))
        .max(3);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(staged_height),
            Constraint::Length(preview_height),
            Constraint::Length(3),
            Constraint::Length(if app.status_message().is_some() { 3 } else { 0 }),
            Constraint::Min(1),
        ])
        .split(inner);

    let mut cursor = None;
    cursor = cursor.or(render_text_row(
        frame,
        rows[0],
        "Type",
        &app.form.commit_type,
        app.focus == Focus::CommitType && matches!(app.mode, Mode::Form),
    ));
    cursor = cursor.or(render_text_row(
        frame,
        rows[1],
        "Scope",
        &app.form.scope,
        app.focus == Focus::Scope,
    ));
    render_toggle_row(
        frame,
        rows[2],
        "Breaking",
        app.form.breaking,
        app.focus == Focus::Breaking,
    );
    cursor = cursor.or(render_text_row(
        frame,
        rows[3],
        &app.subject_indicator(),
        &app.form.message,
        app.focus == Focus::Message,
    ));
    cursor = cursor.or(render_text_row(
        frame,
        rows[4],
        "Body",
        &app.form.body,
        app.focus == Focus::Body,
    ));
    render_footers(frame, rows[5], app);
    cursor = cursor.or(render_text_row(
        frame,
        rows[6],
        "Issue",
        &app.form.issue,
        app.focus == Focus::Issue,
    ));
    render_toggle_row(
        frame,
        rows[7],
        "Sign commit",
        app.sign,
        app.focus == Focus::Sign,
    );
    render_staged_changes(frame, rows[8], app);
    let form_preview_limit = render_preview(
        frame,
        rows[9],
        &app.preview(),
        app.focus == Focus::Preview,
        app.preview_scroll,
    );
    render_submit_row(frame, rows[10], app.focus == Focus::Submit);
    if let Some(status) = app.status_message() {
        render_status(frame, rows[11], status, app.status_is_error());
    }
    render_footer(frame, rows[12]);

    if let Mode::TypePicker(picker) = &app.mode {
        cursor = render_picker(frame, area, picker);
    }
    if let Mode::FooterNamePicker(picker) = &app.mode {
        cursor = render_footer_name_picker(frame, area, picker);
    }
    if let Mode::FooterEditor(editor) = &app.mode {
        cursor = render_footer_editor(frame, area, editor);
    }
    let expanded_preview_limit = if matches!(app.mode, Mode::PreviewExpanded) {
        Some(render_preview_expanded(
            frame,
            area,
            &app.preview(),
            app.preview_scroll,
        ))
    } else {
        None
    };
    if expanded_preview_limit.is_some() {
        cursor = None;
    }
    if matches!(app.mode, Mode::Help(_)) {
        render_help(frame, area);
        cursor = None;
    }

    if let Some(position) = cursor {
        frame.set_cursor_position(position);
    }
    PreviewScrollLimits {
        form: form_preview_limit,
        expanded: expanded_preview_limit,
    }
}

fn render_resize_instruction(frame: &mut Frame, area: Rect) {
    let message = "Terminal too small. Resize to at least 30x8.";
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_compact(frame: &mut Frame, app: &App, area: Rect) -> PreviewScrollLimits {
    let outer = Block::default().borders(Borders::ALL).title(" cocommit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let status_height = u16::from(app.status_message().is_some());
    let preview_height = inner
        .height
        .saturating_sub(COMPACT_FORM_ROWS + status_height + 1)
        .clamp(1, 6);
    let form_height = inner
        .height
        .saturating_sub(preview_height + status_height + 1)
        .clamp(1, COMPACT_FORM_ROWS);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(form_height),
            Constraint::Length(preview_height),
            Constraint::Length(status_height),
            Constraint::Length(1),
        ])
        .split(inner);

    let scroll = focus_index(app.focus).saturating_sub(usize::from(form_height.saturating_sub(1)));
    let mut cursor = None;
    for index in scroll..(scroll + usize::from(form_height)).min(11) {
        let row = Rect::new(
            rows[0].x,
            rows[0].y + (index - scroll) as u16,
            rows[0].width,
            1,
        );
        cursor = cursor.or(render_compact_row(frame, row, app, index));
    }

    let form_preview_limit = render_preview(
        frame,
        rows[1],
        &app.preview(),
        app.focus == Focus::Preview,
        app.preview_scroll,
    );
    if status_height > 0 {
        let status = if app.status_is_error() {
            format!("Error: {}", app.status_message().unwrap_or(""))
        } else {
            app.status_message().unwrap_or("").to_owned()
        };
        frame.render_widget(
            Paragraph::new(status).style(status_style(app.status_is_error())),
            rows[2],
        );
    }
    frame.render_widget(
        Paragraph::new(FOOTER)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        rows[3],
    );

    if let Mode::TypePicker(picker) = &app.mode {
        cursor = render_picker(frame, area, picker);
    }
    if let Mode::FooterNamePicker(picker) = &app.mode {
        cursor = render_footer_name_picker(frame, area, picker);
    }
    if let Mode::FooterEditor(editor) = &app.mode {
        cursor = render_footer_editor(frame, area, editor);
    }
    let expanded_preview_limit = if matches!(app.mode, Mode::PreviewExpanded) {
        Some(render_preview_expanded(
            frame,
            area,
            &app.preview(),
            app.preview_scroll,
        ))
    } else {
        None
    };
    if expanded_preview_limit.is_some() {
        cursor = None;
    }
    if matches!(app.mode, Mode::Help(_)) {
        render_help(frame, area);
        cursor = None;
    }
    if let Some(position) = cursor {
        frame.set_cursor_position(position);
    }
    PreviewScrollLimits {
        form: form_preview_limit,
        expanded: expanded_preview_limit,
    }
}

fn render_compact_row(frame: &mut Frame, area: Rect, app: &App, index: usize) -> Option<Position> {
    let (label, input, toggle, focused) = match index {
        0 => (
            "Type",
            Some(&app.form.commit_type),
            None,
            app.focus == Focus::CommitType,
        ),
        1 => (
            "Scope",
            Some(&app.form.scope),
            None,
            app.focus == Focus::Scope,
        ),
        2 => (
            "Breaking",
            None,
            Some(app.form.breaking),
            app.focus == Focus::Breaking,
        ),
        3 => (
            "Message",
            Some(&app.form.message),
            None,
            app.focus == Focus::Message,
        ),
        4 => ("Body", Some(&app.form.body), None, app.focus == Focus::Body),
        5 => ("Footers", None, None, app.focus == Focus::Footers),
        6 => (
            "Issue",
            Some(&app.form.issue),
            None,
            app.focus == Focus::Issue,
        ),
        7 => (
            "Sign commit",
            None,
            Some(app.sign),
            app.focus == Focus::Sign,
        ),
        8 => ("Staged", None, None, app.focus == Focus::StagedChanges),
        9 => ("Preview", None, None, app.focus == Focus::Preview),
        10 => ("", None, None, app.focus == Focus::Submit),
        _ => return None,
    };
    let style = focused_style(focused);
    let staged_position = if app.staged_changes.files.is_empty() {
        0
    } else {
        app.staged_selected.min(app.staged_changes.files.len() - 1) + 1
    };
    let content = match toggle {
        Some(value) => format!("{label:<10} [{}]", if value { "x" } else { " " }),
        None if index == 10 => "[ Commit ]".to_owned(),
        None if index == 5 => format!(
            "{label:<10} {}/{}  A add  Enter edit  Delete remove",
            if app.form.footers.is_empty() {
                0
            } else {
                app.footer_selected + 1
            },
            app.form.footers.len()
        ),
        None if index == 8 => format!(
            "{label} {}/{} +{} -{} #{}/{}",
            app.included_staged_count(),
            app.staged_changes.files.len(),
            app.staged_changes.insertions,
            app.staged_changes.deletions,
            staged_position,
            app.staged_changes.files.len(),
        ),
        None if index == 9 => format!("{label:<10} scroll {}", app.preview_scroll),
        None => format!("{label:<10}"),
    };
    frame.render_widget(
        Paragraph::new(content)
            .style(style)
            .alignment(if index == 10 {
                Alignment::Center
            } else {
                Alignment::Left
            }),
        area,
    );

    let input = input?;
    let input_x = area.x + 10.min(area.width);
    let input_width = usize::from(area.right().saturating_sub(input_x).max(1));
    let scroll = input.visual_scroll(input_width);
    frame.render_widget(
        Paragraph::new(input.to_string())
            .scroll((0, clamp_scroll(scroll)))
            .style(style),
        Rect::new(input_x, area.y, input_width as u16, 1),
    );
    (matches!(app.mode, Mode::Form)
        && matches!(
            app.focus,
            Focus::Scope | Focus::Message | Focus::Body | Focus::Issue
        )
        && focused)
        .then(|| {
            Position::new(
                input_x
                    + input
                        .visual_cursor()
                        .saturating_sub(scroll)
                        .min(input_width.saturating_sub(1)) as u16,
                area.y,
            )
        })
}

fn focus_index(focus: Focus) -> usize {
    match focus {
        Focus::CommitType => 0,
        Focus::Scope => 1,
        Focus::Breaking => 2,
        Focus::Message => 3,
        Focus::Body => 4,
        Focus::Footers => 5,
        Focus::Issue => 6,
        Focus::Sign => 7,
        Focus::StagedChanges => 8,
        Focus::Preview => 9,
        Focus::Submit => 10,
    }
}

fn render_text_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    input: &Input,
    focused: bool,
) -> Option<Position> {
    let block = field_block(label, focused);
    let input_area = block.inner(area);
    let view = multiline_view(input, input_area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(input.to_string()).scroll((view.vertical_scroll, view.horizontal_scroll)),
        input_area,
    );

    focused.then(|| Position::new(input_area.x + view.column, input_area.y + view.row))
}

struct MultilineView {
    vertical_scroll: u16,
    horizontal_scroll: u16,
    row: u16,
    column: u16,
}

fn multiline_view(input: &Input, area: Rect) -> MultilineView {
    let width = usize::from(area.width.max(1));
    let height = usize::from(area.height.max(1));
    let value = input.to_string();
    let before_cursor = value.chars().take(input.cursor());
    let mut line = 0usize;
    let mut column = 0usize;
    for character in before_cursor {
        if character == '\n' {
            line += 1;
            column = 0;
        } else {
            column += character.width().unwrap_or(0);
        }
    }
    let vertical_scroll = line.saturating_sub(height.saturating_sub(1));
    let horizontal_scroll = column.saturating_sub(width.saturating_sub(1));
    MultilineView {
        vertical_scroll: clamp_scroll(vertical_scroll),
        horizontal_scroll: clamp_scroll(horizontal_scroll),
        row: (line - vertical_scroll).min(height.saturating_sub(1)) as u16,
        column: (column - horizontal_scroll).min(width.saturating_sub(1)) as u16,
    }
}

fn render_toggle_row(frame: &mut Frame, area: Rect, label: &str, value: bool, focused: bool) {
    let marker = if value { "x" } else { " " };
    let block = field_block(label, focused);
    let content = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(format!("[{}]", marker)), content);
}

fn render_submit_row(frame: &mut Frame, area: Rect, focused: bool) {
    render_action_row(frame, area, "Commit", focused);
}

fn render_action_row(frame: &mut Frame, area: Rect, label: &str, focused: bool) {
    let style = focused_style(focused);
    let block = Block::default().borders(Borders::ALL).border_style(style);
    let content = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(format!("[ {label} ]"))
            .alignment(Alignment::Center)
            .style(style),
        content,
    );
}

fn render_preview(frame: &mut Frame, area: Rect, preview: &str, focused: bool, scroll: u16) -> u16 {
    let content = Block::default().borders(Borders::ALL).inner(area);
    let viewport = preview_viewport(preview, content, scroll);
    frame.render_widget(
        Paragraph::new(preview)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " Preview {}/{}{} ",
                        viewport.first_line,
                        viewport.total_lines,
                        if viewport.has_hidden_content {
                            " · Enter expand"
                        } else {
                            ""
                        }
                    ))
                    .border_style(focused_style(focused)),
            )
            .scroll((viewport.scroll, 0))
            .wrap(Wrap { trim: false }),
        area,
    );
    viewport.max_scroll
}

fn render_preview_expanded(frame: &mut Frame, area: Rect, preview: &str, scroll: u16) -> u16 {
    let popup = centered_rect(area, 90, 85);
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Preview ")
        .border_style(focused_style(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let viewport = preview_viewport(preview, rows[0], scroll);
    frame.render_widget(
        Paragraph::new(preview)
            .scroll((viewport.scroll, 0))
            .wrap(Wrap { trim: false }),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(format!(
            "Lines {}-{}/{}  Up/Down scroll  Home/End start/end  Enter/Esc close",
            viewport.first_line, viewport.last_line, viewport.total_lines
        ))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true }),
        rows[1],
    );
    viewport.max_scroll
}

struct PreviewViewport {
    scroll: u16,
    max_scroll: u16,
    first_line: usize,
    last_line: usize,
    total_lines: usize,
    has_hidden_content: bool,
}

fn preview_viewport(preview: &str, area: Rect, requested_scroll: u16) -> PreviewViewport {
    let total_lines = wrapped_line_count(preview, usize::from(area.width.max(1)));
    let visible_lines = usize::from(area.height.max(1));
    let max_scroll = total_lines.saturating_sub(visible_lines);
    let scroll = usize::from(requested_scroll).min(max_scroll);
    let first_line = scroll + 1;
    let last_line = (scroll + visible_lines).min(total_lines);
    PreviewViewport {
        scroll: clamp_scroll(scroll),
        max_scroll: clamp_scroll(max_scroll),
        first_line,
        last_line,
        total_lines,
        has_hidden_content: scroll > 0 || last_line < total_lines,
    }
}

fn wrapped_line_count(value: &str, width: usize) -> usize {
    value
        .split('\n')
        .map(|line| {
            let columns = line
                .chars()
                .map(|character| character.width().unwrap_or(0))
                .sum::<usize>();
            columns.max(1).div_ceil(width.max(1))
        })
        .sum::<usize>()
        .max(1)
}

fn render_footers(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Footers && matches!(app.mode, Mode::Form);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Footers (A add, Enter edit, Delete remove, Ctrl+Up/Down reorder) ")
        .border_style(focused_style(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if app.form.footers.is_empty() {
        frame.render_widget(
            Paragraph::new("No footers. A adds one; Space adds BREAKING CHANGE.")
                .style(focused_style(focused))
                .wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }
    let mut state = ListState::default();
    state.select(Some(app.footer_selected));
    frame.render_stateful_widget(
        List::new(app.form.footers.iter().map(|footer| {
            ListItem::new(format!(
                "{}: {}",
                footer.token,
                footer.value.replace('\n', " / ")
            ))
        }))
        .highlight_symbol("> ")
        .highlight_style(focused_style(focused)),
        inner,
        &mut state,
    );
}

fn render_staged_changes(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::StagedChanges && matches!(app.mode, Mode::Form);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Staged changes ")
        .border_style(focused_style(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let summary = format!(
        "{}/{} included  {}  +{} -{}{}",
        app.included_staged_count(),
        app.staged_changes.files.len(),
        app.staged_changes.status_summary(),
        app.staged_changes.insertions,
        app.staged_changes.deletions,
        if app.staged_changes.binary_files == 0 {
            String::new()
        } else {
            format!("  {} binary", app.staged_changes.binary_files)
        },
    );
    let summary_area = Rect::new(inner.x, inner.y, inner.width, inner.height.min(1));
    frame.render_widget(
        Paragraph::new(summary).style(focused_style(focused)),
        summary_area,
    );

    let list_height = usize::from(inner.height.saturating_sub(1));
    if list_height == 0 {
        return;
    }
    let selected = app
        .staged_selected
        .min(app.staged_changes.files.len().saturating_sub(1));
    let scroll = selected.saturating_sub(list_height.saturating_sub(1));
    let items = app
        .staged_changes
        .files
        .iter()
        .skip(scroll)
        .take(list_height)
        .enumerate()
        .map(|(index, file)| {
            let file_index = scroll + index;
            ListItem::new(format!(
                "{} [{}] {}",
                if file_index == selected { ">" } else { " " },
                if app.staged_file_is_included(file_index) {
                    "x"
                } else {
                    " "
                },
                file.label()
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).style(focused_style(focused)),
        Rect::new(
            inner.x,
            inner.y.saturating_add(1),
            inner.width,
            inner.height.saturating_sub(1),
        ),
    );
}

fn render_status(frame: &mut Frame, area: Rect, status: &str, is_error: bool) {
    frame.render_widget(
        Paragraph::new(status)
            .block(Block::default().borders(Borders::ALL).title(if is_error {
                " Error "
            } else {
                " Status "
            }))
            .style(status_style(is_error))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(FOOTER)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_help(frame: &mut Frame, area: Rect) {
    let width = area.width.saturating_sub(4).min(64);
    let height = area.height.saturating_sub(2).min(16);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(
            "Keyboard help\n\nUp / Down        Move between fields; select footers or staged files\nTab / Shift+Tab  Next / previous field or footer-editor control\nEnter            Type picker; newline in body/footer value; edit footer\nA                Open footer-name picker\nSpace            Toggle Breaking or Sign; add BREAKING CHANGE in Footers\nDelete           Remove selected footer\nCtrl+Up/Down     Reorder selected footer\nCtrl+Enter       Commit form or save footer\nEsc              Close popup or cancel\nCtrl+C           Cancel application\nF1               Open or close help",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(focused_style(true)),
        )
        .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_picker(frame: &mut Frame, area: Rect, picker: &TypePickerState) -> Option<Position> {
    render_choice_picker(
        frame,
        area,
        " Select commit type ",
        &picker.query,
        picker.highlighted,
        picker.choices(),
    )
}

fn render_footer_name_picker(
    frame: &mut Frame,
    area: Rect,
    picker: &FooterNamePickerState,
) -> Option<Position> {
    render_choice_picker(
        frame,
        area,
        " Select footer name ",
        &picker.query,
        picker.highlighted,
        picker.choices(),
    )
}

fn render_choice_picker(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    query: &Input,
    highlighted: usize,
    choices: Vec<TypeChoice>,
) -> Option<Position> {
    let width = area.width.saturating_sub(4).min(60);
    let height = area.height.saturating_sub(2).min(18);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(focused_style(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let width = usize::from(chunks[0].width.max(1));
    let scroll = query.visual_scroll(width);
    frame.render_widget(
        Paragraph::new(query.to_string()).scroll((0, clamp_scroll(scroll))),
        chunks[0],
    );

    let items = choices
        .iter()
        .map(|choice| ListItem::new(choice_label(choice)))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select((!choices.is_empty()).then_some(highlighted));
    frame.render_stateful_widget(
        List::new(items).highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        chunks[1],
        &mut state,
    );

    let column = query
        .visual_cursor()
        .saturating_sub(scroll)
        .min(width.saturating_sub(1)) as u16;
    Some(Position::new(chunks[0].x + column, chunks[0].y))
}

fn render_footer_editor(
    frame: &mut Frame,
    area: Rect,
    editor: &FooterEditorState,
) -> Option<Position> {
    let width = area.width.saturating_sub(4).min(70);
    let height = area.height.saturating_sub(2).min(18);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Footer editor ")
        .border_style(focused_style(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(inner);
    let mut cursor = render_text_row(
        frame,
        rows[0],
        "Footer name",
        &editor.token,
        editor.focus == FooterEditorFocus::Name,
    );
    cursor = cursor.or(render_text_row(
        frame,
        rows[1],
        "Value (Enter newline)",
        &editor.value,
        editor.focus == FooterEditorFocus::Value,
    ));
    render_action_row(
        frame,
        rows[2],
        "Save footer",
        editor.focus == FooterEditorFocus::Save,
    );
    cursor
}

fn choice_label(choice: &TypeChoice) -> String {
    match choice {
        TypeChoice::Standard(value) => value.clone(),
        TypeChoice::CustomQuery(value) => format!("Use \"{}\"", value),
    }
}

fn field_block(label: &str, focused: bool) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(Line::from(format!(" {} ", label)))
        .border_style(focused_style(focused))
}

fn focused_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn status_style(is_error: bool) -> Style {
    Style::default().fg(if is_error { Color::Red } else { Color::Green })
}

fn clamp_scroll(scroll: usize) -> u16 {
    scroll.min(usize::from(u16::MAX)) as u16
}

fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let width = area
        .width
        .saturating_mul(width_percent)
        .saturating_div(100)
        .max(1);
    let height = area
        .height
        .saturating_mul(height_percent)
        .saturating_div(100)
        .max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

#[cfg(test)]
mod tests;
