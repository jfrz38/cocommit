use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{App, Focus, Mode};

use super::components::focused_style;

pub(super) fn render_footers(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus() == Focus::Footers && matches!(app.mode(), Mode::Form);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Footers (A add, Enter edit, Delete remove, Ctrl+Up/Down reorder) ")
        .border_style(focused_style(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if app.form().footers.is_empty() {
        frame.render_widget(
            Paragraph::new("No footers. A adds one; Space adds BREAKING CHANGE.")
                .style(focused_style(focused))
                .wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }
    let mut state = ListState::default();
    state.select(Some(app.footer_selected()));
    frame.render_stateful_widget(
        List::new(app.form().footers.iter().map(|footer| {
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

pub(super) fn render_staged_changes(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus() == Focus::StagedChanges && matches!(app.mode(), Mode::Form);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Staged changes ")
        .border_style(focused_style(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let summary = format!(
        "{}/{} included  {}  +{} -{}{}",
        app.included_staged_count(),
        app.staged_changes().files.len(),
        app.staged_changes().status_summary(),
        app.staged_changes().insertions,
        app.staged_changes().deletions,
        if app.staged_changes().binary_files == 0 {
            String::new()
        } else {
            format!("  {} binary", app.staged_changes().binary_files)
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
        .staged_selected()
        .min(app.staged_changes().files.len().saturating_sub(1));
    let scroll = selected.saturating_sub(list_height.saturating_sub(1));
    let items = app
        .staged_changes()
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
