use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;

use super::components::focused_style;

pub(super) fn render(frame: &mut Frame, area: Rect, app: &App) {
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
        Paragraph::new(text(app))
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

fn text(app: &App) -> String {
    let sections = app.sections();
    let mut lines = vec![
        "Keyboard help".to_owned(),
        String::new(),
        format!(
            "Up / Down        Move between fields{}{}",
            if sections.footers {
                "; select footers"
            } else {
                ""
            },
            if sections.staged_changes {
                " or staged files"
            } else {
                ""
            },
        ),
        format!(
            "Tab / Shift+Tab  Next / previous field{}",
            if sections.footers {
                " or footer-editor control"
            } else {
                ""
            }
        ),
        format!(
            "Enter            Type picker{}; open preview",
            if sections.body {
                "; newline in body"
            } else {
                ""
            }
        ),
        "Space            Toggle Breaking or Sign".to_owned(),
    ];
    if sections.footers {
        lines.extend([
            "A                Open footer-name picker".to_owned(),
            "Space in Footers Add BREAKING CHANGE".to_owned(),
            "Delete           Remove selected footer".to_owned(),
            "Ctrl+Up/Down     Reorder selected footer".to_owned(),
        ]);
    }
    lines.extend([
        "Ctrl+Enter       Commit form or save footer".to_owned(),
        "Esc              Close popup or cancel".to_owned(),
        "Ctrl+C           Cancel application".to_owned(),
        "F1               Open or close help".to_owned(),
    ]);
    lines.join("\n")
}
