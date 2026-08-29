//! Terminal event mapping.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{AppEvent, Edit};

/// Converts terminal input into state-machine events.
pub fn map(event: Event) -> Option<AppEvent> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => map_key(key),
        Event::Paste(value) => Some(AppEvent::Paste(value)),
        Event::Resize(width, height) => Some(AppEvent::Resize(width, height)),
        _ => None,
    }
}

fn map_key(key: KeyEvent) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppEvent::Cancel)
        }
        (KeyCode::Esc, _) => Some(AppEvent::Escape),
        (KeyCode::F(1), _) => Some(AppEvent::Help),
        (KeyCode::Tab, modifiers) if modifiers.contains(KeyModifiers::SHIFT) => {
            Some(AppEvent::BackTab)
        }
        (KeyCode::BackTab, _) => Some(AppEvent::BackTab),
        (KeyCode::Tab, _) => Some(AppEvent::Tab),
        (KeyCode::Enter, modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppEvent::Submit)
        }
        (KeyCode::Enter, _) => Some(AppEvent::Enter),
        (KeyCode::Char(' '), _) => Some(AppEvent::Space),
        (KeyCode::Up, _) => Some(AppEvent::Up),
        (KeyCode::Down, _) => Some(AppEvent::Down),
        (KeyCode::Backspace, _) => Some(AppEvent::Edit(Edit::Backspace)),
        (KeyCode::Delete, _) => Some(AppEvent::Edit(Edit::Delete)),
        (KeyCode::Home, _) => Some(AppEvent::Edit(Edit::Home)),
        (KeyCode::End, _) => Some(AppEvent::Edit(Edit::End)),
        (KeyCode::Char(character), modifiers)
            if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            Some(AppEvent::Edit(Edit::Insert(character)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
