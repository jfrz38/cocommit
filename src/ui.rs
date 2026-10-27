//! Ratatui rendering.

use ratatui::{
    Frame,
    layout::{Alignment, Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
const APP_TITLE: &str = " cocommit ";
const TITLE_GAP: usize = 1;

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

pub(super) fn render_app_frame(frame: &mut Frame, area: Rect, app: &App) -> Rect {
    let outer = Block::default()
        .borders(Borders::ALL)
        .title(APP_TITLE)
        .title(head_title(area.width, app).right_aligned())
        .title_bottom(version_title(area.width, app).right_aligned());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    inner
}

fn head_title(width: u16, app: &App) -> Line<'static> {
    let available = usize::from(width)
        .saturating_sub(2)
        .saturating_sub(APP_TITLE.width())
        .saturating_sub(TITLE_GAP);
    let Some(label) = app.head_label() else {
        return Line::default();
    };
    let available_for_label = available.saturating_sub(2);
    if available_for_label < 4 {
        return Line::default();
    }
    Line::from(format!(" {} ", truncate_label(label, available_for_label)))
}

fn truncate_label(label: &str, max_width: usize) -> String {
    if label.width() <= max_width {
        return label.to_owned();
    }
    let content_width = max_width.saturating_sub(1);
    let mut truncated = String::new();
    let mut width = 0;
    for character in label.chars() {
        let character_width = character.width().unwrap_or(0);
        if width + character_width > content_width {
            break;
        }
        truncated.push(character);
        width += character_width;
    }
    truncated.push('…');
    truncated
}

/// Describes the installed version, highlighting a newer GitHub tag.
fn version_title(width: u16, app: &App) -> Line<'static> {
    let current = env!("CARGO_PKG_VERSION");
    let Some(latest) = app.update_version() else {
        return Line::from(format!(" v{current} "));
    };
    let warning = Style::new().fg(Color::Yellow);
    let announcement = format!(" v{current} → {latest} ");
    let border_corners = 2;
    if usize::from(width) >= announcement.chars().count() + border_corners {
        return Line::from(Span::styled(announcement, warning));
    }
    Line::from(vec![
        Span::raw(format!(" v{current}")),
        Span::styled("!", warning),
        Span::raw(" "),
    ])
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
