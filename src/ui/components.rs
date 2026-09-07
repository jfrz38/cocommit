use ratatui::{
    Frame,
    layout::{Alignment, Position, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_input::Input;
use unicode_width::UnicodeWidthChar;

use crate::{
    app::{App, InputError},
    commit::{
        Capitalization, DraftField, SubjectPolicy, TerminalPunctuation, ValidationError,
        ValidationErrorKind,
    },
};

use super::preview::clamp_scroll;

pub(super) fn status_for(app: &App) -> Option<(&str, bool)> {
    let feedback = app.feedback();
    feedback
        .input_error
        .map(|error| (input_error_message(error), true))
        .or_else(|| {
            feedback
                .validation_error
                .map(|error| (validation_error_message(error), true))
        })
        .or_else(|| {
            feedback
                .operation_status
                .as_deref()
                .map(|message| (message, true))
        })
}

pub(super) fn render_text_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    input: &Input,
    focused: bool,
) -> Option<Position> {
    let block = field_block(label, focused);
    let input_area = block.inner(area);
    let view = multiline_view(input, input_area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(input.to_string()).scroll((view.vertical_scroll, view.horizontal_scroll)),
        input_area,
    );

    focused.then(|| Position::new(input_area.x + view.column, input_area.y + view.row))
}

pub(super) struct MultilineView {
    pub(super) vertical_scroll: u16,
    pub(super) horizontal_scroll: u16,
    pub(super) row: u16,
    pub(super) column: u16,
}

pub(super) fn multiline_view(input: &Input, area: Rect) -> MultilineView {
    let width = usize::from(area.width.max(1));
    let height = usize::from(area.height.max(1));
    let value = input.to_string();
    let before_cursor = value.chars().take(input.cursor());
    let mut line = 0usize;
    let mut column = 0usize;
    for character in before_cursor {
        if character == '\n' {
            line += 1;
            column = 0;
        } else {
            column += character.width().unwrap_or(0);
        }
    }
    let vertical_scroll = line.saturating_sub(height.saturating_sub(1));
    let horizontal_scroll = column.saturating_sub(width.saturating_sub(1));
    MultilineView {
        vertical_scroll: clamp_scroll(vertical_scroll),
        horizontal_scroll: clamp_scroll(horizontal_scroll),
        row: (line - vertical_scroll).min(height.saturating_sub(1)) as u16,
        column: (column - horizontal_scroll).min(width.saturating_sub(1)) as u16,
    }
}

pub(super) fn render_toggle_row(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: bool,
    focused: bool,
) {
    let marker = if value { "x" } else { " " };
    let block = field_block(label, focused);
    let content = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(format!("[{marker}]")), content);
}

pub(super) fn render_submit_row(frame: &mut Frame, area: Rect, focused: bool) {
    render_action_row(frame, area, "Commit", focused);
}

pub(super) fn render_action_row(frame: &mut Frame, area: Rect, label: &str, focused: bool) {
    let style = focused_style(focused);
    let block = Block::default().borders(Borders::ALL).border_style(style);
    let content = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(format!("[ {label} ]"))
            .alignment(Alignment::Center)
            .style(style),
        content,
    );
}

pub(super) fn render_status(frame: &mut Frame, area: Rect, status: &str, is_error: bool) {
    frame.render_widget(
        Paragraph::new(status)
            .block(Block::default().borders(Borders::ALL).title(if is_error {
                " Error "
            } else {
                " Status "
            }))
            .style(status_style(is_error))
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub(super) fn render_footer(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(footer_hint())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub(super) fn footer_hint() -> &'static str {
    "F1 Help  |  Up/Down Navigate  |  Space Toggle  |  Ctrl+Enter Commit  |  Esc Cancel"
}

pub(super) fn subject_indicator(message: &Input, policy: &SubjectPolicy) -> String {
    let length = message.to_string().chars().count();
    let limit = policy
        .max_length
        .map_or_else(|| length.to_string(), |max| format!("{length}/{max}"));
    let capitalization = match policy.capitalization {
        Capitalization::Allow => None,
        Capitalization::Lowercase => Some("lowercase"),
        Capitalization::Uppercase => Some("uppercase"),
    };
    let punctuation = match policy.terminal_punctuation {
        TerminalPunctuation::Allow => None,
        TerminalPunctuation::Forbid => Some("no terminal punctuation"),
        TerminalPunctuation::Require => Some("terminal punctuation required"),
    };
    [
        Some(format!("Subject {limit}")),
        capitalization.map(str::to_owned),
        punctuation.map(str::to_owned),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("; ")
}

pub(super) fn scope_label(has_suggestions: bool) -> &'static str {
    if has_suggestions {
        "Scope (Enter suggestions)"
    } else {
        "Scope"
    }
}

pub(super) fn type_picker_title(types_are_restricted: bool) -> &'static str {
    if types_are_restricted {
        " Allowed commit types "
    } else {
        " Select commit type "
    }
}

pub(super) fn field_block(label: &str, focused: bool) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(Line::from(format!(" {label} ")))
        .border_style(focused_style(focused))
}

pub(super) fn focused_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

pub(super) fn status_style(is_error: bool) -> Style {
    Style::default().fg(if is_error { Color::Red } else { Color::Green })
}

pub(super) fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let width = area
        .width
        .saturating_mul(width_percent)
        .saturating_div(100)
        .max(1);
    let height = area
        .height
        .saturating_mul(height_percent)
        .saturating_div(100)
        .max(1);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn input_error_message(error: InputError) -> &'static str {
    match error {
        InputError::ControlCharacter => "Control characters are not supported",
        InputError::FieldTooLong => "Field is too long",
        InputError::InvalidScopeCharacter => "Scope contains an invalid character",
        InputError::TooManyFooters => "Too many footers",
        InputError::PasteUnsupportedContent => {
            "Paste contains unsupported control characters or is too large"
        }
        InputError::PasteExceedsFieldLimit => "Paste exceeds the field limit",
    }
}

fn validation_error_message(error: ValidationError) -> &'static str {
    match (error.field, error.kind) {
        (DraftField::CommitType, ValidationErrorKind::Required) => "Type is required",
        (DraftField::CommitType, ValidationErrorKind::MustBeSingleLine) => "Type must be one line",
        (DraftField::CommitType, ValidationErrorKind::ContainsWhitespace) => {
            "Type cannot contain whitespace"
        }
        (DraftField::CommitType, ValidationErrorKind::ContainsForbiddenCharacter) => {
            "Type contains an invalid character"
        }
        (DraftField::CommitType, ValidationErrorKind::NotAllowed) => {
            "Type is not allowed by repository policy"
        }
        (DraftField::Scope, ValidationErrorKind::MustBeSingleLine) => "Scope must be one line",
        (DraftField::Scope, ValidationErrorKind::ContainsForbiddenCharacter) => {
            "Scope cannot contain parentheses"
        }
        (DraftField::Message, ValidationErrorKind::Required) => "Message is required",
        (DraftField::Message, ValidationErrorKind::MustBeSingleLine) => "Message must be one line",
        (DraftField::Message, ValidationErrorKind::TooLong) => "Message exceeds repository limit",
        (DraftField::Message, ValidationErrorKind::InvalidCapitalization) => {
            "Message capitalization does not match policy"
        }
        (DraftField::Message, ValidationErrorKind::TerminalPunctuationForbidden) => {
            "Message cannot end with . ! or ?"
        }
        (DraftField::Message, ValidationErrorKind::TerminalPunctuationRequired) => {
            "Message must end with . ! or ?"
        }
        (DraftField::Issue, ValidationErrorKind::InvalidDecimal) => {
            "Issue must be a decimal number"
        }
        (DraftField::Issue, ValidationErrorKind::IntegerOutOfRange) => "Issue is too large",
        (DraftField::Footer(_), ValidationErrorKind::Required) => "Footer value is required",
        (DraftField::Footer(_), ValidationErrorKind::ContainsBlankLine) => {
            "Footer values cannot contain blank lines"
        }
        (DraftField::Footer(_), ValidationErrorKind::ContainsForbiddenCharacter) => {
            "Footer token is invalid"
        }
        (DraftField::Footer(_), ValidationErrorKind::DuplicateSemanticField) => {
            "Breaking change footer is duplicated"
        }
        _ => "Commit message is invalid",
    }
}
