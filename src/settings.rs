//! Application settings shared by configuration and the user interface.

use serde::Deserialize;

/// Portable named colors available for focused controls and picker selections.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccentColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    #[default]
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

/// Editable values resolved before the terminal interface starts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComposerDefaults {
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub body: Option<String>,
    pub issue: Option<String>,
}

/// Optional composer sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSections {
    pub commit_type: bool,
    pub scope: bool,
    pub breaking: bool,
    pub staged_changes: bool,
    pub body: bool,
    pub footers: bool,
    pub issue: bool,
    pub sign: bool,
}

impl Default for UiSections {
    fn default() -> Self {
        Self {
            commit_type: true,
            scope: true,
            breaking: true,
            staged_changes: true,
            body: true,
            footers: true,
            issue: true,
            sign: true,
        }
    }
}
