use ratatui::layout::Rect;

use super::*;

#[test]
fn viewport_clamps_scroll_and_counts_wide_wrapped_lines() {
    let viewport = preview_viewport("header\n界界界界\nfooter", Rect::new(0, 0, 4, 2), 99);

    assert_eq!(viewport.total_lines, 6);
    assert_eq!(viewport.max_scroll, 4);
    assert_eq!(viewport.scroll, 4);
    assert_eq!(viewport.first_line, 5);
    assert_eq!(viewport.last_line, 6);
    assert!(viewport.has_hidden_content);
}

#[test]
fn clamps_scroll_values_that_exceed_terminal_coordinates() {
    assert_eq!(clamp_scroll(usize::MAX), u16::MAX);
}
