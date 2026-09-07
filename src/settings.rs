//! Application settings shared by configuration and the user interface.

/// Optional composer sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSections {
    pub staged_changes: bool,
    pub body: bool,
    pub footers: bool,
    pub issue: bool,
}

impl Default for UiSections {
    fn default() -> Self {
        Self {
            staged_changes: true,
            body: true,
            footers: true,
            issue: true,
        }
    }
}
