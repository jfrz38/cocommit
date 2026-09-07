//! Selection state and invariants for the staged-change list.

use crate::staging::{StagedChanges, StagedFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ToggleResult {
    Changed,
    Empty,
    LastIncluded,
}

#[derive(Debug, Clone, Default)]
pub(super) struct StagedSelection {
    changes: StagedChanges,
    selected: usize,
    included: Vec<bool>,
    enabled: bool,
}

impl StagedSelection {
    pub(super) fn replace(&mut self, changes: StagedChanges, enabled: bool) {
        self.included = if enabled {
            vec![true; changes.files.len()]
        } else {
            Vec::new()
        };
        self.changes = changes;
        self.selected = 0;
        self.enabled = enabled;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.included = if enabled {
            vec![true; self.changes.files.len()]
        } else {
            Vec::new()
        };
    }

    pub(super) fn changes(&self) -> &StagedChanges {
        &self.changes
    }

    pub(super) fn selected(&self) -> usize {
        self.selected
    }

    pub(super) fn select_first(&mut self) {
        self.selected = 0;
    }

    pub(super) fn select_last(&mut self) {
        self.selected = self.changes.files.len().saturating_sub(1);
    }

    pub(super) fn move_selection(&mut self, direction: isize) -> bool {
        let Some(next) = self.selected.checked_add_signed(direction) else {
            return false;
        };
        if next >= self.changes.files.len() {
            return false;
        }
        self.selected = next;
        true
    }

    pub(super) fn is_empty(&self) -> bool {
        self.changes.files.is_empty()
    }

    pub(super) fn is_included(&self, index: usize) -> bool {
        self.enabled && self.included.get(index).copied().unwrap_or(false)
    }

    pub(super) fn included_count(&self) -> usize {
        if !self.enabled {
            return self.changes.files.len();
        }
        self.included.iter().filter(|included| **included).count()
    }

    pub(super) fn excluded_files(&self) -> Vec<StagedFile> {
        if !self.enabled {
            return Vec::new();
        }
        self.changes
            .files
            .iter()
            .zip(&self.included)
            .filter(|&(_, included)| !included)
            .map(|(file, _)| file.clone())
            .collect()
    }

    pub(super) fn toggle_selected(&mut self) -> ToggleResult {
        if self.changes.files.is_empty() {
            return ToggleResult::Empty;
        }
        let selected = self.selected.min(self.changes.files.len() - 1);
        if self.is_included(selected) && self.included_count() <= 1 {
            return ToggleResult::LastIncluded;
        }
        self.included[selected] = !self.included[selected];
        ToggleResult::Changed
    }
}
