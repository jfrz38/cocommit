//! Searchable picker state shared by type, scope, and footer-name overlays.

use tui_input::Input;

use crate::commit::MessagePolicy;

use super::{
    Edit, MAX_FOOTER_TOKEN_LENGTH,
    input::{PasteError, prepare_paste},
};

const DEFAULT_FOOTER_NAMES: &[&str] = &[
    "BREAKING CHANGE",
    "Closes",
    "Fixes",
    "Refs",
    "Co-authored-by",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeChoice {
    Standard(String),
    CustomQuery(String),
}

impl TypeChoice {
    pub(super) fn value(self) -> String {
        match self {
            Self::Standard(value) | Self::CustomQuery(value) => value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypePickerState {
    pub query: Input,
    pub highlighted: usize,
}

impl TypePickerState {
    pub(super) fn new() -> Self {
        Self {
            query: Input::default(),
            highlighted: 0,
        }
    }

    pub fn choices(&self) -> Vec<TypeChoice> {
        self.choices_for(&MessagePolicy::default())
    }

    pub fn choices_for(&self, policy: &MessagePolicy) -> Vec<TypeChoice> {
        choices_for(&self.query, &policy.types, !policy.types_are_restricted)
    }

    pub fn scope_choices_for(&self, policy: &MessagePolicy) -> Vec<TypeChoice> {
        choices_for(&self.query, &policy.scope_suggestions, true)
    }

    pub(super) fn move_highlight(&mut self, choices_len: usize, direction: isize) {
        if choices_len > 0 {
            self.highlighted =
                (self.highlighted as isize + direction).rem_euclid(choices_len as isize) as usize;
        }
    }

    pub(super) fn selected(&self, choices: &[TypeChoice]) -> Option<String> {
        choices
            .get(self.highlighted)
            .cloned()
            .map(TypeChoice::value)
    }

    pub(super) fn edit(
        &mut self,
        edit: Edit,
        max_length: usize,
        rejects_parentheses: bool,
    ) -> Result<(), PickerInputError> {
        edit_input(
            &mut self.query,
            &mut self.highlighted,
            edit,
            max_length,
            rejects_parentheses,
        )
    }

    pub(super) fn paste(
        &mut self,
        value: &str,
        max_length: usize,
        rejects_parentheses: bool,
    ) -> Result<bool, PickerPasteError> {
        let value = prepare_paste(&self.query, value, max_length, false)
            .map_err(PickerPasteError::Paste)?;
        if rejects_parentheses
            && value
                .chars()
                .any(|character| matches!(character, '(' | ')'))
        {
            return Err(PickerPasteError::Input(
                PickerInputError::InvalidScopeCharacter,
            ));
        }
        for character in value.chars() {
            self.edit(Edit::Insert(character), max_length, rejects_parentheses)
                .expect("prepared picker paste must satisfy picker input rules");
        }
        Ok(!value.is_empty())
    }
}

fn edit_input(
    input: &mut Input,
    highlighted: &mut usize,
    edit: Edit,
    max_length: usize,
    rejects_parentheses: bool,
) -> Result<(), PickerInputError> {
    if let Edit::Insert(character) = edit {
        if character.is_control() {
            return Err(PickerInputError::ControlCharacter);
        }
        if rejects_parentheses && matches!(character, '(' | ')') {
            return Err(PickerInputError::InvalidScopeCharacter);
        }
        if input.to_string().chars().count() >= max_length {
            return Err(PickerInputError::TooLong);
        }
    }
    input.handle(edit.request());
    *highlighted = 0;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct FooterNamePickerState {
    pub query: Input,
    pub highlighted: usize,
}

impl FooterNamePickerState {
    pub(super) fn new() -> Self {
        Self {
            query: Input::default(),
            highlighted: 0,
        }
    }

    pub fn choices(&self) -> Vec<TypeChoice> {
        choices_for(
            &self.query,
            &DEFAULT_FOOTER_NAMES
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            true,
        )
    }

    pub(super) fn move_highlight(&mut self, direction: isize) {
        let choices_len = self.choices().len();
        if choices_len > 0 {
            self.highlighted =
                (self.highlighted as isize + direction).rem_euclid(choices_len as isize) as usize;
        }
    }

    pub(super) fn selected(&self) -> Option<String> {
        self.choices()
            .get(self.highlighted)
            .cloned()
            .map(TypeChoice::value)
    }

    pub(super) fn edit(&mut self, edit: Edit) -> Result<(), PickerInputError> {
        edit_input(
            &mut self.query,
            &mut self.highlighted,
            edit,
            MAX_FOOTER_TOKEN_LENGTH,
            false,
        )
    }

    pub(super) fn paste(&mut self, value: &str) -> Result<bool, PickerPasteError> {
        let value = prepare_paste(&self.query, value, MAX_FOOTER_TOKEN_LENGTH, false)
            .map_err(PickerPasteError::Paste)?;
        for character in value.chars() {
            self.edit(Edit::Insert(character))
                .expect("prepared picker paste must satisfy picker input rules");
        }
        Ok(!value.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PickerInputError {
    ControlCharacter,
    InvalidScopeCharacter,
    TooLong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PickerPasteError {
    Paste(PasteError),
    Input(PickerInputError),
}

fn choices_for(query: &Input, values: &[String], allow_custom: bool) -> Vec<TypeChoice> {
    let query = query.to_string();
    let query_lower = query.to_lowercase();
    let mut choices = values
        .iter()
        .filter(|value| value.to_lowercase().starts_with(&query_lower))
        .cloned()
        .map(TypeChoice::Standard)
        .collect::<Vec<_>>();
    if allow_custom && !query.is_empty() && !values.iter().any(|value| value == &query) {
        choices.push(TypeChoice::CustomQuery(query));
    }
    choices
}
