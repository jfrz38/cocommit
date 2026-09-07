//! Event reducers and state mutations for composer interactions.

use tui_input::Input;

use crate::commit::{DraftField, Footer, ValidationErrorKind};

use super::{
    App, AppAction, AppEvent, Edit, Feedback, Focus, FooterEditorFocus, FooterEditorState,
    FooterNamePickerState, InputError, MAX_FOOTERS, MAX_SCOPE_LENGTH, MAX_TYPE_LENGTH, Mode,
    SubmissionIntent, ToggleResult, TypePickerState,
    footer_editor::{FooterEditorInputError, FooterEditorPasteError},
    input::{PasteError, field_limit, prepare_paste},
    picker::{PickerInputError, PickerPasteError},
};

impl App {
    pub(super) fn handle_form(&mut self, event: AppEvent) -> AppAction {
        self.ensure_visible_focus();
        match event {
            AppEvent::Tab => self.focus = self.next_focus(),
            AppEvent::BackTab => self.focus = self.previous_focus(),
            AppEvent::Down if self.focus == Focus::StagedChanges => self.move_staged(1),
            AppEvent::Up if self.focus == Focus::StagedChanges => self.move_staged(-1),
            AppEvent::Down if self.focus == Focus::Footers => self.move_footer_selection(1),
            AppEvent::Up if self.focus == Focus::Footers => self.move_footer_selection(-1),
            AppEvent::Up if self.focus == Focus::Preview && self.preview_state.can_scroll_up() => {
                self.preview_state.scroll_up()
            }
            AppEvent::Down
                if self.focus == Focus::Preview && self.preview_state.can_scroll_down(false) =>
            {
                self.preview_state.scroll_down(false)
            }
            AppEvent::Edit(Edit::Home) if self.focus == Focus::Preview => {
                self.preview_state.scroll_to_start()
            }
            AppEvent::Edit(Edit::End) if self.focus == Focus::Preview => {
                self.preview_state.scroll_to_end(false)
            }
            AppEvent::Down => self.focus = self.next_focus(),
            AppEvent::Up => self.focus = self.previous_focus(),
            AppEvent::Edit(Edit::Home) if self.focus == Focus::StagedChanges => {
                self.staged_selection.select_first();
            }
            AppEvent::Edit(Edit::End) if self.focus == Focus::StagedChanges => {
                self.staged_selection.select_last();
            }
            AppEvent::Enter => match self.focus {
                Focus::CommitType => self.open_picker(),
                Focus::Scope if !self.message_policy.scope_suggestions.is_empty() => {
                    self.open_scope_picker()
                }
                Focus::Scope | Focus::Message | Focus::Issue => self.focus = self.next_focus(),
                Focus::Body => self.edit_form(Edit::Insert('\n')),
                Focus::Footers => {
                    if self.form.footers.is_empty() {
                        self.open_footer_name_picker();
                    } else {
                        self.open_footer_editor(Some(self.footer_selected));
                    }
                }
                Focus::Submit => return self.submit(),
                Focus::Preview => self.mode = Mode::PreviewExpanded,
                Focus::Breaking | Focus::Sign | Focus::StagedChanges => {}
            },
            AppEvent::Submit => return self.submit(),
            AppEvent::Space => match self.focus {
                Focus::Breaking => {
                    self.form.breaking = !self.form.breaking;
                    self.reset_preview_scroll();
                    self.clear_validation_error();
                }
                Focus::Sign => {
                    self.sign = !self.sign;
                    self.clear_validation_error();
                }
                Focus::Scope | Focus::Message | Focus::Body | Focus::Issue => {
                    self.edit_form(Edit::Insert(' '))
                }
                Focus::Footers => self.open_breaking_footer(),
                Focus::StagedChanges => match self.staged_selection.toggle_selected() {
                    ToggleResult::Changed => self.feedback.operation_status = None,
                    ToggleResult::LastIncluded => {
                        self.set_operation_error("Cannot unstage the last staged file")
                    }
                    ToggleResult::Empty => {}
                },
                _ => {}
            },
            AppEvent::MoveFooter(direction) if self.focus == Focus::Footers => {
                self.move_footer(direction)
            }
            AppEvent::Edit(Edit::Insert('a' | 'A')) if self.focus == Focus::Footers => {
                self.open_footer_name_picker()
            }
            AppEvent::Edit(Edit::Delete) if self.focus == Focus::Footers => self.remove_footer(),
            AppEvent::Escape | AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Edit(edit) if self.focus == Focus::CommitType => {
                self.open_picker();
                self.edit_picker(edit);
            }
            AppEvent::Paste(value) if self.focus == Focus::CommitType => {
                self.open_picker();
                self.paste_picker(&value);
            }
            AppEvent::Edit(edit) => self.edit_form(edit),
            AppEvent::Paste(value) => self.paste_form(&value),
            AppEvent::Resize(_, _) | AppEvent::Help | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    pub(super) fn handle_preview_expanded(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape | AppEvent::Enter => self.mode = Mode::Form,
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Submit => {
                self.mode = Mode::Form;
                return self.submit();
            }
            AppEvent::Up => self.preview_state.scroll_up(),
            AppEvent::Down => self.preview_state.scroll_down(true),
            AppEvent::Edit(Edit::Home) => self.preview_state.scroll_to_start(),
            AppEvent::Edit(Edit::End) => self.preview_state.scroll_to_end(true),
            AppEvent::Tab
            | AppEvent::BackTab
            | AppEvent::Space
            | AppEvent::Edit(_)
            | AppEvent::Paste(_)
            | AppEvent::Resize(_, _)
            | AppEvent::Help
            | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    pub(super) fn handle_type_picker(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape => {
                self.mode = Mode::Form;
                self.feedback.input_error = None;
            }
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Enter => self.select_picker_choice(),
            AppEvent::Up => self.move_highlight(-1),
            AppEvent::Down => self.move_highlight(1),
            AppEvent::Edit(edit) => self.edit_picker(edit),
            AppEvent::Paste(value) => self.paste_picker(&value),
            AppEvent::Space => self.edit_picker(Edit::Insert(' ')),
            AppEvent::Tab
            | AppEvent::BackTab
            | AppEvent::Submit
            | AppEvent::Resize(_, _)
            | AppEvent::Help
            | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    pub(super) fn open_picker(&mut self) {
        self.mode = Mode::TypePicker(TypePickerState::new());
    }

    pub(super) fn open_scope_picker(&mut self) {
        self.mode = Mode::ScopePicker(TypePickerState::new());
    }

    pub(super) fn handle_scope_picker(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape => {
                self.mode = Mode::Form;
                self.feedback.input_error = None;
            }
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Enter => self.select_scope_choice(),
            AppEvent::Up => self.move_scope_highlight(-1),
            AppEvent::Down => self.move_scope_highlight(1),
            AppEvent::Edit(edit) => self.edit_scope_picker(edit),
            AppEvent::Paste(value) => self.paste_scope_picker(&value),
            AppEvent::Space => self.edit_scope_picker(Edit::Insert(' ')),
            AppEvent::Tab
            | AppEvent::BackTab
            | AppEvent::Submit
            | AppEvent::Resize(_, _)
            | AppEvent::Help
            | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    pub(super) fn open_footer_name_picker(&mut self) {
        if !self.sections.footers {
            return;
        }
        if self.form.footers.len() >= MAX_FOOTERS {
            self.feedback.input_error = Some(InputError::TooManyFooters);
            return;
        }
        self.mode = Mode::FooterNamePicker(FooterNamePickerState::new());
    }

    pub(super) fn handle_footer_name_picker(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape => {
                self.mode = Mode::Form;
                self.feedback.input_error = None;
            }
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Enter => self.select_footer_name_choice(),
            AppEvent::Up => self.move_footer_name_highlight(-1),
            AppEvent::Down => self.move_footer_name_highlight(1),
            AppEvent::Edit(edit) => self.edit_footer_name_picker(edit),
            AppEvent::Paste(value) => self.paste_footer_name_picker(&value),
            AppEvent::Space => self.edit_footer_name_picker(Edit::Insert(' ')),
            AppEvent::Tab
            | AppEvent::BackTab
            | AppEvent::Submit
            | AppEvent::Resize(_, _)
            | AppEvent::Help
            | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    pub(super) fn open_footer_editor(&mut self, index: Option<usize>) {
        if !self.sections.footers {
            return;
        }
        if index.is_none() && self.form.footers.len() >= MAX_FOOTERS {
            self.feedback.input_error = Some(InputError::TooManyFooters);
            return;
        }
        let footer = index.and_then(|index| self.form.footers.get(index));
        self.mode = Mode::FooterEditor(FooterEditorState::new(index, footer));
    }

    fn open_breaking_footer(&mut self) {
        if let Some(index) = self
            .form
            .footers
            .iter()
            .position(Footer::is_breaking_change)
        {
            self.footer_selected = index;
            self.open_footer_editor(Some(index));
        } else {
            self.open_footer_editor(None);
            if let Mode::FooterEditor(editor) = &mut self.mode {
                editor.set_breaking_change();
            }
        }
    }

    pub(super) fn handle_footer_editor(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Escape => self.mode = Mode::Form,
            AppEvent::Tab => self.move_footer_editor_focus(1),
            AppEvent::BackTab => self.move_footer_editor_focus(-1),
            AppEvent::Enter => {
                let insert_newline =
                    matches!(&self.mode, Mode::FooterEditor(editor) if editor.is_editing_value());
                if insert_newline {
                    self.edit_footer_editor(Edit::Insert('\n'));
                } else {
                    self.save_footer_editor();
                }
            }
            AppEvent::Submit => self.save_footer_editor(),
            AppEvent::Edit(edit) => self.edit_footer_editor(edit),
            AppEvent::Paste(value) => self.paste_footer_editor(&value),
            AppEvent::Space => self.edit_footer_editor(Edit::Insert(' ')),
            AppEvent::Help
            | AppEvent::Resize(_, _)
            | AppEvent::Up
            | AppEvent::Down
            | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    fn move_footer_editor_focus(&mut self, direction: isize) {
        let Mode::FooterEditor(editor) = &mut self.mode else {
            return;
        };
        editor.move_focus(direction);
    }

    fn save_footer_editor(&mut self) {
        let Mode::FooterEditor(editor) = &self.mode else {
            return;
        };
        let index = editor.index;
        let footer = editor.footer();
        if let Some(index) = index {
            self.form.footers[index] = footer;
            self.footer_selected = index;
        } else {
            self.form.footers.push(footer);
            self.footer_selected = self.form.footers.len() - 1;
        }
        self.mode = Mode::Form;
        self.focus = Focus::Footers;
        self.reset_preview_scroll();
        self.clear_validation_error();
    }

    fn edit_footer_editor(&mut self, edit: Edit) {
        let result = match &mut self.mode {
            Mode::FooterEditor(editor) => editor.edit(edit),
            _ => return,
        };
        match result {
            Ok(true) => self.clear_validation_error(),
            Ok(false) => {}
            Err(error) => self.set_footer_editor_input_error(error),
        }
    }

    fn paste_footer_editor(&mut self, value: &str) {
        let result = match &mut self.mode {
            Mode::FooterEditor(editor) => editor.paste(value),
            _ => return,
        };
        match result {
            Ok(true) => self.clear_validation_error(),
            Ok(false) => {}
            Err(FooterEditorPasteError::Paste(error)) => self.set_paste_error(error),
        }
    }

    fn remove_footer(&mut self) {
        if self.footer_selected >= self.form.footers.len() {
            return;
        }
        self.form.footers.remove(self.footer_selected);
        self.footer_selected = self
            .footer_selected
            .min(self.form.footers.len().saturating_sub(1));
        self.reset_preview_scroll();
        self.clear_validation_error();
    }

    fn move_footer(&mut self, direction: isize) {
        let index = self.footer_selected;
        if index >= self.form.footers.len() {
            return;
        }
        let Some(next) = index.checked_add_signed(direction) else {
            return;
        };
        if next >= self.form.footers.len() {
            return;
        }
        self.form.footers.swap(index, next);
        self.footer_selected = next;
        self.reset_preview_scroll();
    }

    fn move_footer_selection(&mut self, direction: isize) {
        if self.form.footers.is_empty() {
            self.focus = if direction < 0 {
                self.previous_focus()
            } else {
                self.next_focus()
            };
            return;
        }
        if direction < 0 {
            if self.footer_selected == 0 {
                self.focus = self.previous_focus();
            } else {
                self.footer_selected -= 1;
            }
        } else if self.footer_selected + 1 < self.form.footers.len() {
            self.footer_selected += 1;
        } else {
            self.focus = self.next_focus();
        }
    }

    fn select_picker_choice(&mut self) {
        let Mode::TypePicker(picker) = &self.mode else {
            return;
        };
        if let Some(value) = picker.selected(&picker.choices_for(&self.message_policy)) {
            self.form.commit_type = Input::new(value);
            self.reset_preview_scroll();
        }
        self.mode = Mode::Form;
        self.focus = self.next_focus();
        self.clear_validation_error();
    }

    fn move_highlight(&mut self, direction: isize) {
        let choices_len = match &self.mode {
            Mode::TypePicker(picker) => picker.choices_for(&self.message_policy).len(),
            _ => return,
        };
        let Mode::TypePicker(picker) = &mut self.mode else {
            return;
        };
        picker.move_highlight(choices_len, direction);
    }

    fn select_scope_choice(&mut self) {
        let choice = match &self.mode {
            Mode::ScopePicker(picker) => picker.selected(&self.scope_choices(picker)),
            _ => return,
        };
        if let Some(value) = choice {
            self.form.scope = Input::new(value);
            self.reset_preview_scroll();
        }
        self.mode = Mode::Form;
        self.focus = self.next_focus();
        self.clear_validation_error();
    }

    fn move_scope_highlight(&mut self, direction: isize) {
        let len = match &self.mode {
            Mode::ScopePicker(picker) => self.scope_choices(picker).len(),
            _ => return,
        };
        if let Mode::ScopePicker(picker) = &mut self.mode {
            picker.move_highlight(len, direction);
        }
    }

    fn select_footer_name_choice(&mut self) {
        let Mode::FooterNamePicker(picker) = &self.mode else {
            return;
        };
        let Some(name) = picker.selected() else {
            return;
        };
        self.open_footer_editor(None);
        if let Mode::FooterEditor(editor) = &mut self.mode {
            editor.token = Input::new(name);
            editor.focus = FooterEditorFocus::Value;
        }
    }

    fn move_footer_name_highlight(&mut self, direction: isize) {
        let Mode::FooterNamePicker(picker) = &mut self.mode else {
            return;
        };
        picker.move_highlight(direction);
    }

    fn edit_form(&mut self, edit: Edit) {
        let input = match self.focus {
            Focus::Scope => Some(&mut self.form.scope),
            Focus::Message => Some(&mut self.form.message),
            Focus::Body => Some(&mut self.form.body),
            Focus::Issue => Some(&mut self.form.issue),
            _ => None,
        };
        if let Some(input) = input {
            if let Edit::Insert(character) = edit {
                if character.is_control() && !(self.focus == Focus::Body && character == '\n') {
                    self.feedback.input_error = Some(InputError::ControlCharacter);
                    return;
                }
                if input.to_string().chars().count() >= field_limit(self.focus) {
                    self.feedback.input_error = Some(InputError::FieldTooLong);
                    return;
                }
            }
            input.handle(edit.request());
            self.reset_preview_scroll();
            self.clear_validation_error();
        }
    }

    fn paste_form(&mut self, value: &str) {
        let input = match self.focus {
            Focus::Scope => &self.form.scope,
            Focus::Message => &self.form.message,
            Focus::Body => &self.form.body,
            Focus::Issue => &self.form.issue,
            _ => return,
        };
        let value = match prepare_paste(
            input,
            value,
            field_limit(self.focus),
            self.focus == Focus::Body,
        ) {
            Ok(value) => value,
            Err(error) => {
                self.set_paste_error(error);
                return;
            }
        };
        for character in value.chars() {
            self.edit_form(Edit::Insert(character));
        }
    }

    fn edit_picker(&mut self, edit: Edit) {
        let Mode::TypePicker(picker) = &mut self.mode else {
            return;
        };
        let result = picker.edit(edit, MAX_TYPE_LENGTH, false);
        self.set_picker_input_result(result);
    }

    fn paste_picker(&mut self, value: &str) {
        let result = match &mut self.mode {
            Mode::TypePicker(picker) => picker.paste(value, MAX_TYPE_LENGTH, false),
            _ => return,
        };
        self.set_picker_paste_result(result);
    }

    fn edit_scope_picker(&mut self, edit: Edit) {
        let Mode::ScopePicker(picker) = &mut self.mode else {
            return;
        };
        let result = picker.edit(edit, MAX_SCOPE_LENGTH, true);
        self.set_picker_input_result(result);
    }

    fn paste_scope_picker(&mut self, value: &str) {
        let result = match &mut self.mode {
            Mode::ScopePicker(picker) => picker.paste(value, MAX_SCOPE_LENGTH, true),
            _ => return,
        };
        self.set_picker_paste_result(result);
    }

    fn edit_footer_name_picker(&mut self, edit: Edit) {
        let Mode::FooterNamePicker(picker) = &mut self.mode else {
            return;
        };
        let result = picker.edit(edit);
        self.set_picker_input_result(result);
    }

    fn paste_footer_name_picker(&mut self, value: &str) {
        let result = match &mut self.mode {
            Mode::FooterNamePicker(picker) => picker.paste(value),
            _ => return,
        };
        self.set_picker_paste_result(result);
    }

    fn submit(&mut self) -> AppAction {
        match self.draft() {
            Ok(draft) => AppAction::Submit(SubmissionIntent {
                draft,
                sign: self.sign,
                excluded_files: self.excluded_staged_files(),
            }),
            Err(errors) => {
                let error = errors[0];
                if let DraftField::Footer(index) = error.field {
                    if !self.sections.footers {
                        self.focus = Focus::Message;
                        self.feedback.validation_error = Some(error);
                        return AppAction::Continue;
                    }
                    self.footer_selected = index.min(self.form.footers.len().saturating_sub(1));
                    self.focus = Focus::Footers;
                    self.open_footer_editor(Some(self.footer_selected));
                    if let Mode::FooterEditor(editor) = &mut self.mode {
                        editor.focus =
                            if error.kind == ValidationErrorKind::ContainsForbiddenCharacter {
                                FooterEditorFocus::Name
                            } else {
                                FooterEditorFocus::Value
                            };
                    }
                } else {
                    let focus = focus_for(error.field);
                    self.focus = if self.visible_focuses().contains(&focus) {
                        focus
                    } else {
                        Focus::Message
                    };
                }
                self.feedback.validation_error = Some(error);
                AppAction::Continue
            }
        }
    }

    fn clear_validation_error(&mut self) {
        self.feedback = Feedback::default();
    }

    fn set_paste_error(&mut self, error: PasteError) {
        self.feedback.input_error = Some(match error {
            PasteError::UnsupportedContent => InputError::PasteUnsupportedContent,
            PasteError::ExceedsFieldLimit => InputError::PasteExceedsFieldLimit,
        });
    }

    fn set_picker_input_result(&mut self, result: Result<(), PickerInputError>) {
        self.feedback.input_error = result.err().map(input_error_from_picker);
    }

    fn set_footer_editor_input_error(&mut self, error: FooterEditorInputError) {
        self.feedback.input_error = Some(input_error_from_footer_editor(error));
    }

    fn set_picker_paste_result(&mut self, result: Result<bool, PickerPasteError>) {
        match result {
            Ok(true) => self.feedback.input_error = None,
            Ok(false) => {}
            Err(PickerPasteError::Paste(error)) => self.set_paste_error(error),
            Err(PickerPasteError::Input(error)) => {
                self.feedback.input_error = Some(input_error_from_picker(error))
            }
        }
    }

    fn reset_preview_scroll(&mut self) {
        self.preview_state.scroll_to_start();
    }

    pub(super) fn preview_is_expanded(&self) -> bool {
        match &self.mode {
            Mode::PreviewExpanded => true,
            Mode::Help(previous) => matches!(previous.as_ref(), Mode::PreviewExpanded),
            _ => false,
        }
    }
}

fn focus_for(field: DraftField) -> Focus {
    match field {
        DraftField::CommitType => Focus::CommitType,
        DraftField::Scope => Focus::Scope,
        DraftField::Message => Focus::Message,
        DraftField::Issue => Focus::Issue,
        DraftField::Footer(_) => Focus::Message,
    }
}

fn input_error_from_picker(error: PickerInputError) -> InputError {
    match error {
        PickerInputError::ControlCharacter => InputError::ControlCharacter,
        PickerInputError::InvalidScopeCharacter => InputError::InvalidScopeCharacter,
        PickerInputError::TooLong => InputError::FieldTooLong,
    }
}

fn input_error_from_footer_editor(error: FooterEditorInputError) -> InputError {
    match error {
        FooterEditorInputError::ControlCharacter => InputError::ControlCharacter,
        FooterEditorInputError::TooLong => InputError::FieldTooLong,
    }
}
