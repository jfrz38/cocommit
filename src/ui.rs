//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Position, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

mod compact;
mod components;
mod expanded;
mod help;
mod modal;
mod preview;
mod staged;

const MIN_WIDTH: u16 = 30;
const MIN_HEIGHT: u16 = 8;
const EXPANDED_WIDTH: u16 = 50;

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

    if area.width >= EXPANDED_WIDTH && area.height >= expanded::minimum_height(app) {
        expanded::render(frame, app, area)
    } else {
        compact::render(frame, app, area)
    }
}

fn render_resize_instruction(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Terminal too small. Resize to at least 30x8.")
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub(super) fn render_app_frame(frame: &mut Frame, area: Rect) -> Rect {
    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" cocommit ")
        .title_bottom(Line::from(format!(" v{} ", env!("CARGO_PKG_VERSION"))).right_aligned());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    inner
}

pub(super) fn render_app_preview(frame: &mut Frame, area: Rect, app: &App) -> u16 {
    preview::render_preview(
        frame,
        area,
        &app.preview(),
        app.focus() == crate::app::Focus::Preview,
        app.preview_state().scroll,
        app.accent_color(),
    )
}

pub(super) fn finish_render(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    cursor: Option<Position>,
    form_preview_limit: u16,
) -> PreviewScrollLimits {
    let (cursor, expanded_preview_limit) = modal::render(frame, area, app, cursor);
    if let Some(position) = cursor {
        frame.set_cursor_position(position);
    }
    PreviewScrollLimits {
        form: form_preview_limit,
        expanded: expanded_preview_limit,
    }
}

#[cfg(test)]
mod tests;
