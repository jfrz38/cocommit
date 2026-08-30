//! Application state and state transitions.

use tui_input::{Input, InputRequest};

use crate::{
    commit::{CommitDraft, DraftField, ValidationError, ValidationErrorKind},
    git::{StagedChanges, StagedFile},
};

pub const STANDARD_TYPES: [&str; 11] = [
    "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore", "revert",
];
pub const MAX_TYPE_LENGTH: usize = 64;
pub const MAX_SCOPE_LENGTH: usize = 128;
pub const MAX_MESSAGE_LENGTH: usize = 512;
pub const MAX_ISSUE_LENGTH: usize = 20;
pub const MAX_PASTE_LENGTH: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    CommitType,
    Scope,
    Breaking,
    Message,
    Issue,
    Sign,
    StagedChanges,
    Submit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    Insert(char),
    Backspace,
    Delete,
    Home,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Tab,
    BackTab,
    Enter,
    Submit,
    Space,
    Escape,
    Cancel,
    Up,
    Down,
    Edit(Edit),
    Paste(String),
    Resize(u16, u16),
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Continue,
    Cancel,
    Submit,
}

#[derive(Debug, Clone)]
pub struct FormState {
    pub commit_type: Input,
    pub scope: Input,
    pub breaking: bool,
    pub message: Input,
    pub issue: Input,
}

impl FormState {
    fn draft(&self) -> Result<CommitDraft, Vec<ValidationError>> {
        CommitDraft::from_raw(
            self.commit_type.to_string(),
            Some(self.scope.to_string()),
            self.breaking,
            self.message.to_string(),
            Some(self.issue.to_string()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct TypePickerState {
    pub query: Input,
    pub highlighted: usize,
}

impl TypePickerState {
    pub fn choices(&self) -> Vec<TypeChoice> {
        let query = self.query.to_string();
        let query_lower = query.to_lowercase();
        let mut choices = STANDARD_TYPES
            .iter()
            .filter(|commit_type| commit_type.starts_with(&query_lower))
            .map(|commit_type| TypeChoice::Standard((*commit_type).to_owned()))
            .collect::<Vec<_>>();

        if !query.is_empty() && !STANDARD_TYPES.contains(&query.as_str()) {
            choices.push(TypeChoice::CustomQuery(query));
        }

        choices
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeChoice {
    Standard(String),
    CustomQuery(String),
}

#[derive(Debug, Clone)]
pub enum Mode {
    Form,
    TypePicker(TypePickerState),
    Help(Box<Mode>),
}

#[derive(Debug, Clone)]
pub struct App {
    pub form: FormState,
    pub focus: Focus,
    pub mode: Mode,
    pub sign: bool,
    pub staged_changes: StagedChanges,
    pub staged_selected: usize,
    staged_included: Vec<bool>,
    pub validation_error: Option<ValidationError>,
    pub input_error: Option<&'static str>,
    operation_status: Option<OperationStatus>,
}

#[derive(Debug, Clone)]
struct OperationStatus {
    message: String,
}

impl App {
    pub fn new(sign: bool) -> Self {
        Self {
            form: FormState {
                commit_type: Input::new("feat".to_owned()),
                scope: Input::default(),
                breaking: false,
                message: Input::default(),
                issue: Input::default(),
            },
            focus: Focus::CommitType,
            mode: Mode::Form,
            sign,
            staged_changes: StagedChanges::default(),
            staged_selected: 0,
            staged_included: Vec::new(),
            validation_error: None,
            input_error: None,
            operation_status: None,
        }
    }

    pub fn draft(&self) -> Result<CommitDraft, Vec<ValidationError>> {
        self.form.draft()
    }

    pub fn set_staged_changes(&mut self, staged_changes: StagedChanges) {
        self.staged_changes = staged_changes;
        self.staged_selected = 0;
        self.staged_included = vec![true; self.staged_changes.files.len()];
    }

    pub fn staged_file_is_included(&self, index: usize) -> bool {
        self.staged_included.get(index).copied().unwrap_or(false)
    }

    pub fn included_staged_count(&self) -> usize {
        self.staged_included
            .iter()
            .filter(|included| **included)
            .count()
    }

    pub fn excluded_staged_files(&self) -> Vec<StagedFile> {
        self.staged_changes
            .files
            .iter()
            .zip(&self.staged_included)
            .filter_map(|(file, included)| (!included).then(|| file.clone()))
            .collect()
    }

    fn set_operation_error(&mut self, message: impl Into<String>) {
        self.operation_status = Some(OperationStatus {
            message: message.into(),
        });
    }

    /// Renders the current form state without requiring it to be valid for submission.
    pub fn preview(&self) -> String {
        let issue = CommitDraft::parse_issue(&self.form.issue.to_string())
            .ok()
            .flatten();
        CommitDraft::new(
            self.form.commit_type.to_string(),
            Some(self.form.scope.to_string()),
            self.form.breaking,
            self.form.message.to_string(),
            issue,
        )
        .render_message()
    }

    /// Returns a concise message suitable for the form status area.
    pub fn validation_message(&self) -> Option<&'static str> {
        self.input_error.or(self
            .validation_error
            .map(|error| match (error.field, error.kind) {
                (DraftField::CommitType, ValidationErrorKind::Required) => "Type is required",
                (DraftField::CommitType, ValidationErrorKind::MustBeSingleLine) => {
                    "Type must be one line"
                }
                (DraftField::CommitType, ValidationErrorKind::ContainsWhitespace) => {
                    "Type cannot contain whitespace"
                }
                (DraftField::CommitType, ValidationErrorKind::ContainsForbiddenCharacter) => {
                    "Type contains an invalid character"
                }
                (DraftField::Scope, ValidationErrorKind::MustBeSingleLine) => {
                    "Scope must be one line"
                }
                (DraftField::Scope, ValidationErrorKind::ContainsForbiddenCharacter) => {
                    "Scope cannot contain parentheses"
                }
                (DraftField::Message, ValidationErrorKind::Required) => "Message is required",
                (DraftField::Message, ValidationErrorKind::MustBeSingleLine) => {
                    "Message must be one line"
                }
                (DraftField::Issue, ValidationErrorKind::InvalidDecimal) => {
                    "Issue must be a decimal number"
                }
                (DraftField::Issue, ValidationErrorKind::IntegerOutOfRange) => "Issue is too large",
                _ => "Invalid value",
            }))
    }

    pub fn status_message(&self) -> Option<&str> {
        self.validation_message().or_else(|| {
            self.operation_status
                .as_ref()
                .map(|status| status.message.as_str())
        })
    }

    pub fn status_is_error(&self) -> bool {
        self.validation_message().is_some() || self.operation_status.is_some()
    }

    pub fn handle(&mut self, event: AppEvent) -> AppAction {
        if matches!(event, AppEvent::Help) {
            self.toggle_help();
            return AppAction::Continue;
        }
        if matches!(event, AppEvent::Escape) && matches!(self.mode, Mode::Help(_)) {
            self.close_help();
            return AppAction::Continue;
        }
        match &mut self.mode {
            Mode::Form => self.handle_form(event),
            Mode::TypePicker(_) => self.handle_picker(event),
            Mode::Help(_previous) => match event {
                AppEvent::Cancel => AppAction::Cancel,
                _ => AppAction::Continue,
            },
        }
    }

    fn toggle_help(&mut self) {
        self.mode = match std::mem::replace(&mut self.mode, Mode::Form) {
            Mode::Help(previous) => *previous,
            previous => Mode::Help(Box::new(previous)),
        };
    }

    fn close_help(&mut self) {
        if let Mode::Help(previous) = std::mem::replace(&mut self.mode, Mode::Form) {
            self.mode = *previous;
        }
    }

    fn handle_form(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Tab => self.focus = self.focus.next(),
            AppEvent::BackTab => self.focus = self.focus.previous(),
            AppEvent::Down if self.focus == Focus::StagedChanges => self.move_staged(1),
            AppEvent::Up if self.focus == Focus::StagedChanges => self.move_staged(-1),
            AppEvent::Down => self.focus = self.focus.next(),
            AppEvent::Up => self.focus = self.focus.previous(),
            AppEvent::Edit(Edit::Home) if self.focus == Focus::StagedChanges => {
                self.staged_selected = 0;
            }
            AppEvent::Edit(Edit::End) if self.focus == Focus::StagedChanges => {
                self.staged_selected = self.staged_changes.files.len().saturating_sub(1);
            }
            AppEvent::Enter => match self.focus {
                Focus::CommitType => self.open_picker(),
                Focus::Scope => self.focus = Focus::Breaking,
                Focus::Message => self.focus = Focus::Issue,
                Focus::Issue => self.focus = Focus::Sign,
                Focus::Submit => return self.submit(),
                Focus::Breaking | Focus::Sign | Focus::StagedChanges => {}
            },
            AppEvent::Submit => return self.submit(),
            AppEvent::Space => match self.focus {
                Focus::Breaking => {
                    self.form.breaking = !self.form.breaking;
                    self.clear_validation_error();
                }
                Focus::Sign => {
                    self.sign = !self.sign;
                    self.clear_validation_error();
                }
                Focus::Scope | Focus::Message | Focus::Issue => self.edit_form(Edit::Insert(' ')),
                Focus::StagedChanges if self.staged_changes.files.is_empty() => {}
                Focus::StagedChanges
                    if self.staged_file_is_included(self.staged_selected)
                        && self.included_staged_count() <= 1 =>
                {
                    self.set_operation_error("Cannot unstage the last staged file");
                }
                Focus::StagedChanges => {
                    let selected = self
                        .staged_selected
                        .min(self.staged_changes.files.len() - 1);
                    self.staged_included[selected] = !self.staged_included[selected];
                    self.operation_status = None;
                }
                _ => {}
            },
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
            AppEvent::Resize(_, _) | AppEvent::Help => {}
        }
        AppAction::Continue
    }

    fn handle_picker(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape => {
                self.mode = Mode::Form;
                self.input_error = None;
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
            | AppEvent::Help => {}
        }
        AppAction::Continue
    }

    fn open_picker(&mut self) {
        self.mode = Mode::TypePicker(TypePickerState {
            query: Input::default(),
            highlighted: 0,
        });
    }

    fn select_picker_choice(&mut self) {
        let Mode::TypePicker(picker) = &self.mode else {
            return;
        };
        let choice = picker.choices().get(picker.highlighted).cloned();
        match choice {
            Some(TypeChoice::Standard(value)) | Some(TypeChoice::CustomQuery(value)) => {
                self.form.commit_type = Input::new(value);
            }
            None => {}
        }
        self.mode = Mode::Form;
        self.focus = Focus::Scope;
        self.clear_validation_error();
    }

    fn move_highlight(&mut self, direction: isize) {
        let Mode::TypePicker(picker) = &mut self.mode else {
            return;
        };
        let len = picker.choices().len();
        if len == 0 {
            return;
        }
        picker.highlighted =
            (picker.highlighted as isize + direction).rem_euclid(len as isize) as usize;
    }

    fn edit_form(&mut self, edit: Edit) {
        let input = match self.focus {
            Focus::Scope => Some(&mut self.form.scope),
            Focus::Message => Some(&mut self.form.message),
            Focus::Issue => Some(&mut self.form.issue),
            _ => None,
        };
        if let Some(input) = input {
            if let Edit::Insert(character) = edit {
                if character.is_control() {
                    self.input_error = Some("Control characters are not supported");
                    return;
                }
                if input.to_string().chars().count() >= field_limit(self.focus) {
                    self.input_error = Some("Field is too long");
                    return;
                }
            }
            input.handle(edit.request());
            self.clear_validation_error();
        }
    }

    fn paste_form(&mut self, value: &str) {
        let Ok(value) = sanitize_paste(value) else {
            self.input_error =
                Some("Paste contains unsupported control characters or is too large");
            return;
        };
        let existing_length = match self.focus {
            Focus::Scope => self.form.scope.to_string().chars().count(),
            Focus::Message => self.form.message.to_string().chars().count(),
            Focus::Issue => self.form.issue.to_string().chars().count(),
            _ => return,
        };
        if existing_length + value.chars().count() > field_limit(self.focus) {
            self.input_error = Some("Paste exceeds the field limit");
            return;
        }
        for character in value.chars() {
            self.edit_form(Edit::Insert(character));
        }
    }

    fn edit_picker(&mut self, edit: Edit) {
        let Mode::TypePicker(picker) = &mut self.mode else {
            return;
        };
        if let Edit::Insert(character) = edit {
            if character.is_control() {
                self.input_error = Some("Control characters are not supported");
                return;
            }
            if picker.query.to_string().chars().count() >= MAX_TYPE_LENGTH {
                self.input_error = Some("Field is too long");
                return;
            }
        }
        picker.query.handle(edit.request());
        picker.highlighted = 0;
        self.input_error = None;
    }

    fn paste_picker(&mut self, value: &str) {
        let Ok(value) = sanitize_paste(value) else {
            self.input_error =
                Some("Paste contains unsupported control characters or is too large");
            return;
        };
        let current_length = match &self.mode {
            Mode::TypePicker(picker) => picker.query.to_string().chars().count(),
            _ => return,
        };
        if current_length + value.chars().count() > MAX_TYPE_LENGTH {
            self.input_error = Some("Paste exceeds the field limit");
            return;
        }
        for character in value.chars() {
            self.edit_picker(Edit::Insert(character));
        }
    }

    fn submit(&mut self) -> AppAction {
        match self.draft() {
            Ok(_) => AppAction::Submit,
            Err(errors) => {
                let error = errors[0];
                self.focus = focus_for(error.field);
                self.validation_error = Some(error);
                AppAction::Continue
            }
        }
    }

    fn clear_validation_error(&mut self) {
        self.validation_error = None;
        self.input_error = None;
        self.operation_status = None;
    }

    fn move_staged(&mut self, direction: isize) {
        if self.staged_changes.files.is_empty() {
            self.focus = if direction < 0 {
                Focus::Sign
            } else {
                Focus::Submit
            };
            return;
        }
        let last = self.staged_changes.files.len() - 1;
        if direction < 0 && self.staged_selected == 0 {
            self.focus = Focus::Sign;
        } else if direction > 0 && self.staged_selected == last {
            self.focus = Focus::Submit;
        } else {
            self.staged_selected =
                (self.staged_selected as isize + direction).clamp(0, last as isize) as usize;
        }
    }
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Self::CommitType => Self::Scope,
            Self::Scope => Self::Breaking,
            Self::Breaking => Self::Message,
            Self::Message => Self::Issue,
            Self::Issue => Self::Sign,
            Self::Sign => Self::StagedChanges,
            Self::StagedChanges => Self::Submit,
            Self::Submit => Self::CommitType,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::CommitType => Self::Submit,
            Self::Scope => Self::CommitType,
            Self::Breaking => Self::Scope,
            Self::Message => Self::Breaking,
            Self::Issue => Self::Message,
            Self::Sign => Self::Issue,
            Self::StagedChanges => Self::Sign,
            Self::Submit => Self::StagedChanges,
        }
    }
}

impl Edit {
    fn request(self) -> InputRequest {
        match self {
            Self::Insert(character) => InputRequest::InsertChar(character),
            Self::Backspace => InputRequest::DeletePrevChar,
            Self::Delete => InputRequest::DeleteNextChar,
            Self::Home => InputRequest::GoToStart,
            Self::End => InputRequest::GoToEnd,
        }
    }
}

fn focus_for(field: DraftField) -> Focus {
    match field {
        DraftField::CommitType => Focus::CommitType,
        DraftField::Scope => Focus::Scope,
        DraftField::Message => Focus::Message,
        DraftField::Issue => Focus::Issue,
    }
}

fn field_limit(focus: Focus) -> usize {
    match focus {
        Focus::Scope => MAX_SCOPE_LENGTH,
        Focus::Message => MAX_MESSAGE_LENGTH,
        Focus::Issue => MAX_ISSUE_LENGTH,
        Focus::CommitType => MAX_TYPE_LENGTH,
        Focus::Breaking | Focus::Sign | Focus::StagedChanges | Focus::Submit => 0,
    }
}

fn sanitize_paste(value: &str) -> Result<String, ()> {
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

#[cfg(test)]
mod tests;
