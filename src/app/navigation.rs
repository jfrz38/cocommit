//! Focus and staged-list navigation for the composer screen.

use super::{App, Focus};

impl App {
    pub(super) fn move_staged(&mut self, direction: isize) {
        if self.staged_selection.is_empty() || !self.staged_selection.move_selection(direction) {
            self.focus = if direction < 0 {
                self.previous_focus()
            } else {
                self.next_focus()
            };
        }
    }

    pub(super) fn next_focus(&self) -> Focus {
        let focuses = self.visible_focuses();
        let index = focuses
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        focuses[(index + 1) % focuses.len()]
    }

    pub(super) fn previous_focus(&self) -> Focus {
        let focuses = self.visible_focuses();
        let index = focuses
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        focuses[(index + focuses.len() - 1) % focuses.len()]
    }

    pub(super) fn ensure_visible_focus(&mut self) {
        if !self.visible_focuses().contains(&self.focus) {
            self.focus = Focus::CommitType;
            self.mode = super::Mode::Form;
        }
    }
}
