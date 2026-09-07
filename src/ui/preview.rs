//! Preview rendering and viewport calculations.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthChar;

use super::components::{centered_rect, focused_style};

pub(super) fn render_preview(
    frame: &mut Frame,
    area: Rect,
    preview: &str,
    focused: bool,
    scroll: u16,
) -> u16 {
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

pub(super) fn render_preview_expanded(
    frame: &mut Frame,
    area: Rect,
    preview: &str,
    scroll: u16,
) -> u16 {
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

pub(super) struct PreviewViewport {
    pub(super) scroll: u16,
    pub(super) max_scroll: u16,
    pub(super) first_line: usize,
    pub(super) last_line: usize,
    pub(super) total_lines: usize,
    pub(super) has_hidden_content: bool,
}

pub(super) fn preview_viewport(
    preview: &str,
    area: Rect,
    requested_scroll: u16,
) -> PreviewViewport {
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

pub(super) fn clamp_scroll(scroll: usize) -> u16 {
    scroll.min(usize::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests;
