use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};

use super::*;

#[test]
fn maps_only_key_press_events() {
    let mut release = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    release.kind = KeyEventKind::Release;
    assert_eq!(map(Event::Key(release)), None);

    let mut repeat = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    repeat.kind = KeyEventKind::Repeat;
    assert_eq!(map(Event::Key(repeat)), None);
    assert_eq!(
        map(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE
        ))),
        Some(AppEvent::Enter)
    );
}

#[test]
fn maps_cancellation_navigation_and_editing() {
    assert_eq!(
        map(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        ))),
        Some(AppEvent::Cancel)
    );
    assert_eq!(
        map(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))),
        Some(AppEvent::Escape)
    );
    assert_eq!(
        map(Event::Key(KeyEvent::new(
            KeyCode::BackTab,
            KeyModifiers::NONE
        ))),
        Some(AppEvent::BackTab)
    );
    assert_eq!(
        map(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::NONE
        ))),
        Some(AppEvent::Edit(Edit::Insert('x')))
    );
    assert_eq!(
        map(Event::Key(KeyEvent::new(
            KeyCode::Delete,
            KeyModifiers::NONE
        ))),
        Some(AppEvent::Edit(Edit::Delete))
    );
}

#[test]
fn maps_paste_and_resize() {
    assert_eq!(
        map(Event::Paste("hello".to_owned())),
        Some(AppEvent::Paste("hello".to_owned()))
    );
    assert_eq!(map(Event::Resize(120, 40)), Some(AppEvent::Resize(120, 40)));
}

#[test]
fn ignores_mouse_and_focus_events() {
    let mouse = MouseEvent {
        kind: MouseEventKind::Moved,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(map(Event::Mouse(mouse)), None);
    assert_eq!(map(Event::FocusGained), None);
}
