//! Commit-message conventions applied during validation and rendering.

use serde::Deserialize;

pub const DEFAULT_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];

/// Repository message conventions resolved from defaults and configuration layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagePolicy {
    pub types: Vec<String>,
    pub types_are_restricted: bool,
    pub scope_suggestions: Vec<String>,
    pub subject: SubjectPolicy,
    pub issue: IssuePolicy,
}

impl Default for MessagePolicy {
    fn default() -> Self {
        Self {
            types: DEFAULT_TYPES.map(str::to_owned).to_vec(),
            types_are_restricted: false,
            scope_suggestions: Vec::new(),
            subject: SubjectPolicy::default(),
            issue: IssuePolicy::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectPolicy {
    pub max_length: Option<usize>,
    pub capitalization: Capitalization,
    pub terminal_punctuation: TerminalPunctuation,
}

impl Default for SubjectPolicy {
    fn default() -> Self {
        Self {
            max_length: None,
            capitalization: Capitalization::Allow,
            terminal_punctuation: TerminalPunctuation::Allow,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Capitalization {
    Allow,
    Lowercase,
    Uppercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalPunctuation {
    Allow,
    Forbid,
    Require,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuePolicy {
    pub prefix: String,
    pub style: IssueStyle,
}

impl Default for IssuePolicy {
    fn default() -> Self {
        Self {
            prefix: "#".to_owned(),
            style: IssueStyle::Parenthesized,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueStyle {
    Parenthesized,
    Plain,
}
