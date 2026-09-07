//! State and input rules for the footer editing overlay.

use tui_input::Input;

use crate::commit::Footer;

use super::{
    Edit, MAX_FOOTER_TOKEN_LENGTH, MAX_FOOTER_VALUE_LENGTH,
    input::{PasteError, prepare_paste},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterEditorFocus {
    Name,
    Value,
    Save,
}

#[derive(Debug, Clone)]
pub struct FooterEditorState {
    pub index: Option<usize>,
    pub token: Input,
    pub value: Input,
    pub focus: FooterEditorFocus,
}

impl FooterEditorState {
    pub(super) fn new(index: Option<usize>, footer: Option<&Footer>) -> Self {
        Self {
            index,
            token: Input::new(footer.map_or_else(String::new, |footer| footer.token.clone())),
            value: Input::new(footer.map_or_else(String::new, |footer| footer.value.clone())),
            focus: FooterEditorFocus::Name,
        }
    }

    pub(super) fn is_editing_value(&self) -> bool {
        self.focus == FooterEditorFocus::Value
    }

    pub(super) fn set_breaking_change(&mut self) {
        self.token = Input::new("BREAKING CHANGE".to_owned());
        self.focus = FooterEditorFocus::Value;
    }

    pub(super) fn move_focus(&mut self, direction: isize) {
        self.focus = match (self.focus, direction.is_negative()) {
            (FooterEditorFocus::Name, false) | (FooterEditorFocus::Save, true) => {
                FooterEditorFocus::Value
            }
            (FooterEditorFocus::Value, false) => FooterEditorFocus::Save,
            (FooterEditorFocus::Value, true) => FooterEditorFocus::Name,
            (FooterEditorFocus::Save, false) => FooterEditorFocus::Name,
            (FooterEditorFocus::Name, true) => FooterEditorFocus::Save,
        };
    }

    pub(super) fn footer(&self) -> Footer {
        Footer::new(self.token.to_string(), self.value.to_string())
    }

    pub(super) fn edit(&mut self, edit: Edit) -> Result<bool, FooterEditorInputError> {
        let (input, limit) = match self.focus {
            FooterEditorFocus::Name => (&mut self.token, MAX_FOOTER_TOKEN_LENGTH),
            FooterEditorFocus::Value => (&mut self.value, MAX_FOOTER_VALUE_LENGTH),
            FooterEditorFocus::Save => return Ok(false),
        };
        if let Edit::Insert(character) = edit {
            if character.is_control() && character != '\n' {
                return Err(FooterEditorInputError::ControlCharacter);
            }
            if input.to_string().chars().count() >= limit {
                return Err(FooterEditorInputError::TooLong);
            }
        }
        input.handle(edit.request());
        Ok(true)
    }

    pub(super) fn paste(&mut self, value: &str) -> Result<bool, FooterEditorPasteError> {
        let (input, limit, multiline) = match self.focus {
            FooterEditorFocus::Name => (&self.token, MAX_FOOTER_TOKEN_LENGTH, false),
            FooterEditorFocus::Value => (&self.value, MAX_FOOTER_VALUE_LENGTH, true),
            FooterEditorFocus::Save => return Ok(false),
        };
        let value =
            prepare_paste(input, value, limit, multiline).map_err(FooterEditorPasteError::Paste)?;
        for character in value.chars() {
            self.edit(Edit::Insert(character))
                .expect("prepared footer paste must satisfy footer input rules");
        }
        Ok(!value.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FooterEditorInputError {
    ControlCharacter,
    TooLong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FooterEditorPasteError {
    Paste(PasteError),
}
