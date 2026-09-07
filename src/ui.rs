//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Paragraph, Wrap},
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
const EXPANDED_HEIGHT: u16 = 39;
const EXPANDED_HEIGHT_WITH_STATUS: u16 = 42;

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

    let expanded_height = if components::status_for(app).is_some() {
        EXPANDED_HEIGHT_WITH_STATUS
    } else {
        EXPANDED_HEIGHT
    };
    if area.width >= EXPANDED_WIDTH && area.height >= expanded_height {
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

#[cfg(test)]
mod tests;
