//! Input limits and sanitization shared by form and modal editors.

use tui_input::Input;

use super::{
    Focus, MAX_BODY_LENGTH, MAX_ISSUE_LENGTH, MAX_MESSAGE_LENGTH, MAX_PASTE_LENGTH,
    MAX_SCOPE_LENGTH, MAX_TYPE_LENGTH,
};

pub(super) fn field_limit(focus: Focus) -> usize {
    match focus {
        Focus::Scope => MAX_SCOPE_LENGTH,
        Focus::Message => MAX_MESSAGE_LENGTH,
        Focus::Body => MAX_BODY_LENGTH,
        Focus::Issue => MAX_ISSUE_LENGTH,
        Focus::CommitType => MAX_TYPE_LENGTH,
        Focus::Breaking
        | Focus::Footers
        | Focus::Sign
        | Focus::StagedChanges
        | Focus::Preview
        | Focus::Submit => 0,
    }
}

pub(super) fn sanitize_paste(value: &str) -> Result<String, ()> {
    if value.chars().count() > MAX_PASTE_LENGTH {
        return Err(());
    }
    let mut sanitized = String::new();
    let mut previous_was_line_break = false;
    for character in value.chars() {
        if matches!(character, '\r' | '\n') {
            if !previous_was_line_break {
                sanitized.push(' ');
            }
            previous_was_line_break = true;
        } else if character.is_control() {
            return Err(());
        } else {
            sanitized.push(character);
            previous_was_line_break = false;
        }
    }
    Ok(sanitized)
}

pub(super) fn sanitize_multiline_paste(value: &str) -> Result<String, ()> {
    if value.chars().count() > MAX_PASTE_LENGTH {
        return Err(());
    }
    let mut sanitized = String::new();
    let mut previous_was_cr = false;
    for character in value.chars() {
        match character {
            '\r' => {
                sanitized.push('\n');
                previous_was_cr = true;
            }
            '\n' if previous_was_cr => previous_was_cr = false,
            '\n' => sanitized.push('\n'),
            character if character.is_control() => return Err(()),
            character => {
                sanitized.push(character);
                previous_was_cr = false;
            }
        }
    }
    Ok(sanitized)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PasteError {
    UnsupportedContent,
    ExceedsFieldLimit,
}

/// Sanitizes and bounds a paste before mutating the target input.
pub(super) fn prepare_paste(
    input: &Input,
    value: &str,
    max_length: usize,
    multiline: bool,
) -> Result<String, PasteError> {
    let value = if multiline {
        sanitize_multiline_paste(value)
    } else {
        sanitize_paste(value)
    }
    .map_err(|_| PasteError::UnsupportedContent)?;
    if input.to_string().chars().count() + value.chars().count() > max_length {
        return Err(PasteError::ExceedsFieldLimit);
    }
    Ok(value)
}
