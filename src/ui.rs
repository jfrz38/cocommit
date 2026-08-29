//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use tui_input::Input;

use crate::app::{App, Focus, Mode, TypeChoice, TypePickerState};

const MIN_WIDTH: u16 = 30;
const MIN_HEIGHT: u16 = 8;
const EXPANDED_WIDTH: u16 = 50;
const EXPANDED_HEIGHT: u16 = 30;
const FOOTER: &str = "F1 Help  Up/Down Navigate  Space Toggle  Ctrl+Enter Commit  Esc Cancel";

/// Draws the complete application state.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_resize_instruction(frame, area);
        return;
    }

    if area.width >= EXPANDED_WIDTH && area.height >= EXPANDED_HEIGHT {
        render_expanded(frame, app, area);
    } else {
        render_compact(frame, app, area);
    }
}

fn render_expanded(frame: &mut Frame, app: &App, area: Rect) {
    let outer = Block::default().borders(Borders::ALL).title(" cocommit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
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
        "Message",
        &app.form.message,
        app.focus == Focus::Message,
    ));
    cursor = cursor.or(render_text_row(
        frame,
        rows[4],
        "Issue",
        &app.form.issue,
        app.focus == Focus::Issue,
    ));
    render_toggle_row(
        frame,
        rows[5],
        "Sign (-S)",
        app.sign,
        app.focus == Focus::Sign,
    );
    render_submit_row(frame, rows[6], app.focus == Focus::Submit);
    render_preview(frame, rows[7], &app.preview());
    render_status(frame, rows[8], app.validation_message());
    render_footer(frame, rows[9]);

    if let Mode::TypePicker(picker) = &app.mode {
        cursor = render_picker(frame, area, picker);
    }
    if matches!(app.mode, Mode::Help(_)) {
        render_help(frame, area);
        cursor = None;
    }

    if let Some(position) = cursor {
        frame.set_cursor_position(position);
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

fn render_compact(frame: &mut Frame, app: &App, area: Rect) {
    let outer = Block::default().borders(Borders::ALL).title(" cocommit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let status_height = u16::from(app.validation_message().is_some());
    let preview_height = 1;
    let form_height = inner
        .height
        .saturating_sub(preview_height + status_height + 1)
        .max(1);
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
    for index in scroll..(scroll + usize::from(form_height)).min(7) {
        let row = Rect::new(
            rows[0].x,
            rows[0].y + (index - scroll) as u16,
            rows[0].width,
            1,
        );
        cursor = cursor.or(render_compact_row(frame, row, app, index));
    }

    frame.render_widget(
        Paragraph::new(format!("Preview: {}", app.preview())).wrap(Wrap { trim: true }),
        rows[1],
    );
    if status_height > 0 {
        frame.render_widget(
            Paragraph::new(app.validation_message().unwrap_or(""))
                .style(Style::default().fg(Color::Red)),
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
    if matches!(app.mode, Mode::Help(_)) {
        render_help(frame, area);
        cursor = None;
    }
    if let Some(position) = cursor {
        frame.set_cursor_position(position);
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
        4 => (
            "Issue",
            Some(&app.form.issue),
            None,
            app.focus == Focus::Issue,
        ),
        5 => ("Sign (-S)", None, Some(app.sign), app.focus == Focus::Sign),
        6 => ("", None, None, app.focus == Focus::Submit),
        _ => return None,
    };
    let style = focused_style(focused);
    let content = match toggle {
        Some(value) => format!("{label:<10} [{}]", if value { "x" } else { " " }),
        None if index == 6 => "[ Commit ]".to_owned(),
        None => format!("{label:<10}"),
    };
    frame.render_widget(
        Paragraph::new(content)
            .style(style)
            .alignment(if index == 6 {
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
            .scroll((0, scroll as u16))
            .style(style),
        Rect::new(input_x, area.y, input_width as u16, 1),
    );
    (matches!(app.mode, Mode::Form)
        && matches!(app.focus, Focus::Scope | Focus::Message | Focus::Issue)
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
        Focus::Issue => 4,
        Focus::Sign => 5,
        Focus::Submit => 6,
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
    let width = usize::from(input_area.width.max(1));
    let scroll = input.visual_scroll(width);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(input.to_string()).scroll((0, scroll as u16)),
        input_area,
    );

    focused.then(|| {
        let column = input
            .visual_cursor()
            .saturating_sub(scroll)
            .min(width.saturating_sub(1)) as u16;
        Position::new(input_area.x + column, input_area.y)
    })
}

fn render_toggle_row(frame: &mut Frame, area: Rect, label: &str, value: bool, focused: bool) {
    let marker = if value { "x" } else { " " };
    let block = field_block(label, focused);
    let content = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(format!("[{}]", marker)), content);
}

fn render_submit_row(frame: &mut Frame, area: Rect, focused: bool) {
    let style = focused_style(focused);
    let block = Block::default().borders(Borders::ALL).border_style(style);
    let content = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new("[ Commit ]")
            .alignment(Alignment::Center)
            .style(style),
        content,
    );
}

fn render_preview(frame: &mut Frame, area: Rect, preview: &str) {
    frame.render_widget(
        Paragraph::new(preview)
            .block(Block::default().borders(Borders::ALL).title(" Preview "))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_status(frame: &mut Frame, area: Rect, status: Option<&str>) {
    frame.render_widget(
        Paragraph::new(status.unwrap_or(""))
            .block(Block::default().borders(Borders::ALL).title(" Status "))
            .style(Style::default().fg(Color::Red))
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
            "Keyboard help\n\nUp / Down        Move between fields\nTab / Shift+Tab  Next / previous field\nEnter            Open or select Type\nSpace            Toggle Breaking or Sign\nCtrl+Enter       Commit the current form\nEsc              Close help or cancel\nCtrl+C           Cancel application\nF1               Open or close help",
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
        .title(" Select commit type ")
        .border_style(focused_style(true));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let width = usize::from(chunks[0].width.max(1));
    let scroll = picker.query.visual_scroll(width);
    frame.render_widget(
        Paragraph::new(picker.query.to_string()).scroll((0, scroll as u16)),
        chunks[0],
    );

    let choices = picker.choices();
    let items = choices
        .iter()
        .map(|choice| ListItem::new(choice_label(choice)))
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select((!choices.is_empty()).then_some(picker.highlighted));
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

    let column = picker
        .query
        .visual_cursor()
        .saturating_sub(scroll)
        .min(width.saturating_sub(1)) as u16;
    Some(Position::new(chunks[0].x + column, chunks[0].y))
}

fn choice_label(choice: &TypeChoice) -> String {
    match choice {
        TypeChoice::Standard(value) => value.clone(),
        TypeChoice::Custom => "custom...".to_owned(),
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

#[cfg(test)]
mod tests;
