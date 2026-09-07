use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
};

use crate::app::{App, Focus, Mode};

use super::{
    PreviewScrollLimits,
    components::{
        render_footer, render_status, render_submit_row, render_text_row, render_toggle_row,
        scope_label, status_for, subject_indicator,
    },
    modal,
    preview::render_preview,
    staged::{render_footers, render_staged_changes},
};

pub(super) fn render(frame: &mut Frame, app: &App, area: Rect) -> PreviewScrollLimits {
    let outer = Block::default().borders(Borders::ALL).title(" cocommit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let sections = app.sections();
    let omitted_form_height = u16::from(!sections.body) * 4
        + u16::from(!sections.footers) * 4
        + u16::from(!sections.issue) * 3;
    let staged_height = sections.staged_changes.then(|| {
        inner
            .height
            .saturating_sub(33 - omitted_form_height + 3 * u16::from(status_for(app).is_some()))
            .clamp(4, 9)
    });
    let preview_height = inner
        .height
        .saturating_sub(
            30 - omitted_form_height - 4 * u16::from(!sections.staged_changes)
                + staged_height.unwrap_or(0)
                + 3 * u16::from(status_for(app).is_some()),
        )
        .max(3);
    let mut constraints = vec![
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ];
    if sections.body {
        constraints.push(Constraint::Length(4));
    }
    if sections.footers {
        constraints.push(Constraint::Length(4));
    }
    if sections.issue {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Length(3));
    if let Some(staged_height) = staged_height {
        constraints.push(Constraint::Length(staged_height));
    }
    constraints.extend([
        Constraint::Length(preview_height),
        Constraint::Length(3),
        Constraint::Length(if status_for(app).is_some() { 3 } else { 0 }),
        Constraint::Min(1),
    ]);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let mut cursor = None;
    cursor = cursor.or(render_text_row(
        frame,
        rows[0],
        "Type",
        &app.form().commit_type,
        app.focus() == Focus::CommitType && matches!(app.mode(), Mode::Form),
    ));
    cursor = cursor.or(render_text_row(
        frame,
        rows[1],
        scope_label(app.has_scope_suggestions()),
        &app.form().scope,
        app.focus() == Focus::Scope,
    ));
    render_toggle_row(
        frame,
        rows[2],
        "Breaking",
        app.form().breaking,
        app.focus() == Focus::Breaking,
    );
    cursor = cursor.or(render_text_row(
        frame,
        rows[3],
        &subject_indicator(&app.form().message, app.subject_policy()),
        &app.form().message,
        app.focus() == Focus::Message,
    ));
    let mut row = 4;
    if sections.body {
        cursor = cursor.or(render_text_row(
            frame,
            rows[row],
            "Body",
            &app.form().body,
            app.focus() == Focus::Body,
        ));
        row += 1;
    }
    if sections.footers {
        render_footers(frame, rows[row], app);
        row += 1;
    }
    if sections.issue {
        cursor = cursor.or(render_text_row(
            frame,
            rows[row],
            "Issue",
            &app.form().issue,
            app.focus() == Focus::Issue,
        ));
        row += 1;
    }
    render_toggle_row(
        frame,
        rows[row],
        "Sign commit",
        app.sign(),
        app.focus() == Focus::Sign,
    );
    row += 1;
    if sections.staged_changes {
        render_staged_changes(frame, rows[row], app);
        row += 1;
    }
    let form_preview_limit = render_preview(
        frame,
        rows[row],
        &app.preview(),
        app.focus() == Focus::Preview,
        app.preview_state().scroll,
    );
    row += 1;
    render_submit_row(frame, rows[row], app.focus() == Focus::Submit);
    row += 1;
    if let Some((status, is_error)) = status_for(app) {
        render_status(frame, rows[row], status, is_error);
    }
    row += 1;
    render_footer(frame, rows[row]);

    let (cursor, expanded_preview_limit) = modal::render(frame, area, app, cursor);
    if let Some(position) = cursor {
        frame.set_cursor_position(position);
    }
    PreviewScrollLimits {
        form: form_preview_limit,
        expanded: expanded_preview_limit,
    }
}
