use ratatui::{
    Frame,
    layout::{Alignment, Position, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_input::Input;

use crate::app::{App, Focus, Mode};

use super::{
    PreviewScrollLimits,
    components::{focused_style, footer_hint, scope_label, status_for, status_style},
    modal,
    preview::{clamp_scroll, render_preview},
};

pub(super) fn render(frame: &mut Frame, app: &App, area: Rect) -> PreviewScrollLimits {
    let outer = Block::default().borders(Borders::ALL).title(" cocommit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let status = status_for(app);
    let status_height = u16::from(status.is_some());
    let focus_count = app.visible_focuses().len() as u16;
    let preview_height = inner
        .height
        .saturating_sub(focus_count + status_height + 1)
        .clamp(1, 6);
    let form_height = inner
        .height
        .saturating_sub(preview_height + status_height + 1)
        .clamp(1, focus_count);
    let rows = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(form_height),
            ratatui::layout::Constraint::Length(preview_height),
            ratatui::layout::Constraint::Length(status_height),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(inner);

    let focuses = app.visible_focuses();
    let focus_index = focuses
        .iter()
        .position(|focus| *focus == app.focus())
        .unwrap_or(0);
    let scroll = focus_index.saturating_sub(usize::from(form_height.saturating_sub(1)));
    let mut cursor = None;
    for (offset, focus) in focuses
        .iter()
        .enumerate()
        .skip(scroll)
        .take(usize::from(form_height))
    {
        let row = Rect::new(
            rows[0].x,
            rows[0].y + (offset - scroll) as u16,
            rows[0].width,
            1,
        );
        cursor = cursor.or(render_row(frame, row, app, *focus));
    }

    let form_preview_limit = render_preview(
        frame,
        rows[1],
        &app.preview(),
        app.focus() == Focus::Preview,
        app.preview_state().scroll,
    );
    if let Some((message, is_error)) = status {
        let status = if is_error {
            format!("Error: {message}")
        } else {
            message.to_owned()
        };
        frame.render_widget(
            Paragraph::new(status).style(status_style(is_error)),
            rows[2],
        );
    }
    frame.render_widget(
        Paragraph::new(footer_hint())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        rows[3],
    );

    let (cursor, expanded_preview_limit) = modal::render(frame, area, app, cursor);
    if let Some(position) = cursor {
        frame.set_cursor_position(position);
    }
    PreviewScrollLimits {
        form: form_preview_limit,
        expanded: expanded_preview_limit,
    }
}

fn render_row(frame: &mut Frame, area: Rect, app: &App, focus: Focus) -> Option<Position> {
    let (label, input, toggle, focused) = match focus {
        Focus::CommitType => (
            "Type",
            Some(&app.form().commit_type),
            None,
            app.focus() == focus,
        ),
        Focus::Scope => (
            scope_label(app.has_scope_suggestions()),
            Some(&app.form().scope),
            None,
            app.focus() == focus,
        ),
        Focus::Breaking => (
            "Breaking",
            None,
            Some(app.form().breaking),
            app.focus() == focus,
        ),
        Focus::Message => (
            "Message",
            Some(&app.form().message),
            None,
            app.focus() == focus,
        ),
        Focus::Body => ("Body", Some(&app.form().body), None, app.focus() == focus),
        Focus::Footers => ("Footers", None, None, app.focus() == focus),
        Focus::Issue => ("Issue", Some(&app.form().issue), None, app.focus() == focus),
        Focus::Sign => ("Sign commit", None, Some(app.sign()), app.focus() == focus),
        Focus::StagedChanges => ("Staged", None, None, app.focus() == focus),
        Focus::Preview => ("Preview", None, None, app.focus() == focus),
        Focus::Submit => ("", None, None, app.focus() == focus),
    };
    let style = focused_style(focused);
    let staged_position = if app.staged_changes().files.is_empty() {
        0
    } else {
        app.staged_selected()
            .min(app.staged_changes().files.len() - 1)
            + 1
    };
    let content = match toggle {
        Some(value) => format!("{label:<10} [{}]", if value { "x" } else { " " }),
        None if focus == Focus::Submit => "[ Commit ]".to_owned(),
        None if focus == Focus::Footers => format!(
            "{label:<10} {}/{}  A add  Enter edit  Delete remove",
            if app.form().footers.is_empty() {
                0
            } else {
                app.footer_selected() + 1
            },
            app.form().footers.len()
        ),
        None if focus == Focus::StagedChanges => format!(
            "{label} {}/{} +{} -{} #{}/{}",
            app.included_staged_count(),
            app.staged_changes().files.len(),
            app.staged_changes().insertions,
            app.staged_changes().deletions,
            staged_position,
            app.staged_changes().files.len(),
        ),
        None if focus == Focus::Preview => {
            format!("{label:<10} scroll {}", app.preview_state().scroll)
        }
        None => format!("{label:<10}"),
    };
    frame.render_widget(
        Paragraph::new(content)
            .style(style)
            .alignment(if focus == Focus::Submit {
                Alignment::Center
            } else {
                Alignment::Left
            }),
        area,
    );

    let input = input?;
    render_input(frame, area, app, focus, input, style, focused)
}

fn render_input(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    focus: Focus,
    input: &Input,
    style: ratatui::style::Style,
    focused: bool,
) -> Option<Position> {
    let input_x = area.x + 10.min(area.width);
    let input_width = usize::from(area.right().saturating_sub(input_x).max(1));
    let scroll = input.visual_scroll(input_width);
    frame.render_widget(
        Paragraph::new(input.to_string())
            .scroll((0, clamp_scroll(scroll)))
            .style(style),
        Rect::new(input_x, area.y, input_width as u16, 1),
    );
    (matches!(app.mode(), Mode::Form)
        && matches!(
            focus,
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
