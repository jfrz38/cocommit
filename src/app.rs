//! Application state and state transitions.

use tui_input::{Input, InputRequest};

use crate::{
    commit::{CommitDraft, DraftField, Footer, ValidationError, ValidationErrorKind},
    config::{Capitalization, MessagePolicy, TerminalPunctuation, UiSections},
    git::{StagedChanges, StagedFile},
};

pub use crate::config::DEFAULT_TYPES as STANDARD_TYPES;
pub const MAX_TYPE_LENGTH: usize = 64;
pub const MAX_SCOPE_LENGTH: usize = 128;
pub const MAX_MESSAGE_LENGTH: usize = 512;
pub const MAX_ISSUE_LENGTH: usize = 20;
pub const MAX_BODY_LENGTH: usize = 4096;
pub const MAX_FOOTER_TOKEN_LENGTH: usize = 64;
pub const MAX_FOOTER_VALUE_LENGTH: usize = 2048;
pub const MAX_FOOTERS: usize = 32;
pub const MAX_PASTE_LENGTH: usize = 4096;
const DEFAULT_FOOTER_NAMES: &[&str] = &[
    "BREAKING CHANGE",
    "Closes",
    "Fixes",
    "Refs",
    "Co-authored-by",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    CommitType,
    Scope,
    Breaking,
    Message,
    Body,
    Footers,
    Issue,
    Sign,
    StagedChanges,
    Preview,
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
    MoveFooter(isize),
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
    pub body: Input,
    pub issue: Input,
    pub footers: Vec<Footer>,
}

impl FormState {
    fn draft(
        &self,
        sections: UiSections,
        policy: &MessagePolicy,
    ) -> Result<CommitDraft, Vec<ValidationError>> {
        CommitDraft::from_raw(
            self.commit_type.to_string(),
            Some(self.scope.to_string()),
            self.breaking,
            self.message.to_string(),
            sections.issue.then(|| self.issue.to_string()),
            sections.body.then(|| self.body.to_string()),
            if sections.footers {
                self.footers.clone()
            } else {
                Vec::new()
            },
        )
        .and_then(|draft| {
            draft.validate_with_policy(policy)?;
            Ok(draft)
        })
    }
}

#[derive(Debug, Clone)]
pub struct TypePickerState {
    pub query: Input,
    pub highlighted: usize,
}

impl TypePickerState {
    pub fn choices(&self) -> Vec<TypeChoice> {
        self.choices_for(&MessagePolicy::default())
    }

    pub fn choices_for(&self, policy: &MessagePolicy) -> Vec<TypeChoice> {
        let query = self.query.to_string();
        let query_lower = query.to_lowercase();
        let mut choices = policy
            .types
            .iter()
            .filter(|commit_type| commit_type.to_lowercase().starts_with(&query_lower))
            .cloned()
            .map(TypeChoice::Standard)
            .collect::<Vec<_>>();

        if !policy.types_are_restricted
            && !query.is_empty()
            && !policy.types.iter().any(|commit_type| commit_type == &query)
        {
            choices.push(TypeChoice::CustomQuery(query));
        }

        choices
    }

    pub fn scope_choices_for(&self, policy: &MessagePolicy) -> Vec<TypeChoice> {
        let query = self.query.to_string();
        let query_lower = query.to_lowercase();
        let mut choices = policy
            .scope_suggestions
            .iter()
            .filter(|scope| scope.to_lowercase().starts_with(&query_lower))
            .cloned()
            .map(TypeChoice::Standard)
            .collect::<Vec<_>>();
        if !query.is_empty() && !policy.scope_suggestions.iter().any(|scope| scope == &query) {
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
pub struct FooterNamePickerState {
    pub query: Input,
    pub highlighted: usize,
}

impl FooterNamePickerState {
    pub fn choices(&self) -> Vec<TypeChoice> {
        let query = self.query.to_string();
        let query_lower = query.to_lowercase();
        let mut choices = DEFAULT_FOOTER_NAMES
            .iter()
            .filter(|name| name.to_lowercase().starts_with(&query_lower))
            .map(|name| TypeChoice::Standard((*name).to_owned()))
            .collect::<Vec<_>>();
        if !query.is_empty()
            && !DEFAULT_FOOTER_NAMES
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&query))
        {
            choices.push(TypeChoice::CustomQuery(query));
        }
        choices
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Form,
    TypePicker(TypePickerState),
    ScopePicker(TypePickerState),
    FooterNamePicker(FooterNamePickerState),
    FooterEditor(FooterEditorState),
    PreviewExpanded,
    Help(Box<Mode>),
}

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

#[derive(Debug, Clone)]
pub struct App {
    pub form: FormState,
    pub focus: Focus,
    pub mode: Mode,
    pub sign: bool,
    pub sections: UiSections,
    pub staged_changes: StagedChanges,
    pub staged_selected: usize,
    pub footer_selected: usize,
    pub preview_scroll: u16,
    preview_scroll_limit: u16,
    expanded_preview_scroll_limit: u16,
    message_policy: MessagePolicy,
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
                body: Input::default(),
                issue: Input::default(),
                footers: Vec::new(),
            },
            focus: Focus::CommitType,
            mode: Mode::Form,
            sign,
            sections: UiSections::default(),
            staged_changes: StagedChanges::default(),
            staged_selected: 0,
            footer_selected: 0,
            preview_scroll: 0,
            preview_scroll_limit: 0,
            expanded_preview_scroll_limit: 0,
            message_policy: MessagePolicy::default(),
            staged_included: Vec::new(),
            validation_error: None,
            input_error: None,
            operation_status: None,
        }
    }

    pub fn with_message_policy(mut self, policy: &MessagePolicy) -> Self {
        self.message_policy = policy.clone();
        if policy.types_are_restricted && !policy.types.contains(&self.form.commit_type.to_string())
        {
            self.form.commit_type = Input::new(policy.types.first().cloned().unwrap_or_default());
        }
        self
    }

    pub fn with_sections(mut self, sections: UiSections) -> Self {
        self.sections = sections;
        if !sections.body {
            self.form.body = Input::default();
        }
        if !sections.footers {
            self.form.footers.clear();
        }
        if !sections.issue {
            self.form.issue = Input::default();
        }
        self.staged_included.clear();
        self.ensure_visible_focus();
        self
    }

    pub fn visible_focuses(&self) -> Vec<Focus> {
        [
            Some(Focus::CommitType),
            Some(Focus::Scope),
            Some(Focus::Breaking),
            Some(Focus::Message),
            self.sections.body.then_some(Focus::Body),
            self.sections.footers.then_some(Focus::Footers),
            self.sections.issue.then_some(Focus::Issue),
            Some(Focus::Sign),
            self.sections.staged_changes.then_some(Focus::StagedChanges),
            Some(Focus::Preview),
            Some(Focus::Submit),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn subject_indicator(&self) -> String {
        let length = self.form.message.to_string().chars().count();
        let limit = self
            .message_policy
            .subject
            .max_length
            .map_or_else(|| length.to_string(), |max| format!("{length}/{max}"));
        let capitalization = match self.message_policy.subject.capitalization {
            Capitalization::Allow => None,
            Capitalization::Lowercase => Some("lowercase"),
            Capitalization::Uppercase => Some("uppercase"),
        };
        let punctuation = match self.message_policy.subject.terminal_punctuation {
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

    pub fn scope_label(&self) -> &'static str {
        if self.message_policy.scope_suggestions.is_empty() {
            "Scope"
        } else {
            "Scope (Enter suggestions)"
        }
    }

    pub fn type_picker_title(&self) -> &'static str {
        if self.message_policy.types_are_restricted {
            " Allowed commit types "
        } else {
            " Select commit type "
        }
    }

    pub fn type_choices(&self, picker: &TypePickerState) -> Vec<TypeChoice> {
        picker.choices_for(&self.message_policy)
    }

    pub fn scope_choices(&self, picker: &TypePickerState) -> Vec<TypeChoice> {
        picker.scope_choices_for(&self.message_policy)
    }

    pub fn draft(&self) -> Result<CommitDraft, Vec<ValidationError>> {
        self.form.draft(self.sections, &self.message_policy)
    }

    pub fn set_staged_changes(&mut self, staged_changes: StagedChanges) {
        self.staged_changes = staged_changes;
        self.staged_selected = 0;
        self.staged_included = if self.sections.staged_changes {
            vec![true; self.staged_changes.files.len()]
        } else {
            Vec::new()
        };
    }

    /// Synchronizes preview scroll limits calculated from the current terminal viewport.
    pub fn set_preview_scroll_limits(&mut self, form_limit: u16, expanded_limit: Option<u16>) {
        self.preview_scroll_limit = form_limit;
        if let Some(expanded_limit) = expanded_limit {
            self.expanded_preview_scroll_limit = expanded_limit;
        }
        self.preview_scroll = self.preview_scroll.min(self.active_preview_scroll_limit());
    }

    pub fn staged_file_is_included(&self, index: usize) -> bool {
        self.sections.staged_changes && self.staged_included.get(index).copied().unwrap_or(false)
    }

    pub fn included_staged_count(&self) -> usize {
        if !self.sections.staged_changes {
            return self.staged_changes.files.len();
        }
        self.staged_included
            .iter()
            .filter(|included| **included)
            .count()
    }

    pub fn excluded_staged_files(&self) -> Vec<StagedFile> {
        if !self.sections.staged_changes {
            return Vec::new();
        }
        self.staged_changes
            .files
            .iter()
            .zip(&self.staged_included)
            .filter(|&(_, included)| !included)
            .map(|(file, _)| file.clone())
            .collect()
    }

    fn set_operation_error(&mut self, message: impl Into<String>) {
        self.operation_status = Some(OperationStatus {
            message: message.into(),
        });
    }

    /// Renders the current form state without requiring it to be valid for submission.
    pub fn preview(&self) -> String {
        let issue = self
            .sections
            .issue
            .then(|| {
                CommitDraft::parse_issue(&self.form.issue.to_string())
                    .ok()
                    .flatten()
            })
            .flatten();
        CommitDraft::new(
            self.form.commit_type.to_string(),
            Some(self.form.scope.to_string()),
            self.form.breaking,
            self.form.message.to_string(),
            issue,
            self.sections.body.then(|| self.form.body.to_string()),
            if self.sections.footers {
                self.form.footers.clone()
            } else {
                Vec::new()
            },
        )
        .render_message_with_policy(&self.message_policy)
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
                (DraftField::CommitType, ValidationErrorKind::NotAllowed) => {
                    "Type is not allowed by repository policy"
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
                (DraftField::Message, ValidationErrorKind::TooLong) => {
                    "Message exceeds repository limit"
                }
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
                (DraftField::Footer(_), ValidationErrorKind::Required) => {
                    "Footer value is required"
                }
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
            Mode::TypePicker(_) => self.handle_type_picker(event),
            Mode::ScopePicker(_) => self.handle_scope_picker(event),
            Mode::FooterNamePicker(_) => self.handle_footer_name_picker(event),
            Mode::FooterEditor(_) => self.handle_footer_editor(event),
            Mode::PreviewExpanded => self.handle_preview_expanded(event),
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
        self.ensure_visible_focus();
        match event {
            AppEvent::Tab => self.focus = self.next_focus(),
            AppEvent::BackTab => self.focus = self.previous_focus(),
            AppEvent::Down if self.focus == Focus::StagedChanges => self.move_staged(1),
            AppEvent::Up if self.focus == Focus::StagedChanges => self.move_staged(-1),
            AppEvent::Down if self.focus == Focus::Footers => self.move_footer_selection(1),
            AppEvent::Up if self.focus == Focus::Footers => self.move_footer_selection(-1),
            AppEvent::Up if self.focus == Focus::Preview && self.preview_scroll > 0 => {
                self.scroll_preview(-1)
            }
            AppEvent::Down
                if self.focus == Focus::Preview
                    && self.preview_scroll < self.preview_scroll_limit =>
            {
                self.scroll_preview(1)
            }
            AppEvent::Edit(Edit::Home) if self.focus == Focus::Preview => self.preview_scroll = 0,
            AppEvent::Edit(Edit::End) if self.focus == Focus::Preview => {
                self.preview_scroll = self.preview_scroll_limit
            }
            AppEvent::Down => self.focus = self.next_focus(),
            AppEvent::Up => self.focus = self.previous_focus(),
            AppEvent::Edit(Edit::Home) if self.focus == Focus::StagedChanges => {
                self.staged_selected = 0;
            }
            AppEvent::Edit(Edit::End) if self.focus == Focus::StagedChanges => {
                self.staged_selected = self.staged_changes.files.len().saturating_sub(1);
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

    fn handle_preview_expanded(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape | AppEvent::Enter => self.mode = Mode::Form,
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Submit => {
                self.mode = Mode::Form;
                return self.submit();
            }
            AppEvent::Up => self.scroll_preview(-1),
            AppEvent::Down => self.scroll_preview(1),
            AppEvent::Edit(Edit::Home) => self.preview_scroll = 0,
            AppEvent::Edit(Edit::End) => self.preview_scroll = self.expanded_preview_scroll_limit,
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

    fn handle_type_picker(&mut self, event: AppEvent) -> AppAction {
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
            | AppEvent::Help
            | AppEvent::MoveFooter(_) => {}
        }
        AppAction::Continue
    }

    fn open_picker(&mut self) {
        self.mode = Mode::TypePicker(TypePickerState {
            query: Input::default(),
            highlighted: 0,
        });
    }

    fn open_scope_picker(&mut self) {
        self.mode = Mode::ScopePicker(TypePickerState {
            query: Input::default(),
            highlighted: 0,
        });
    }

    fn handle_scope_picker(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape => {
                self.mode = Mode::Form;
                self.input_error = None;
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

    fn open_footer_name_picker(&mut self) {
        if !self.sections.footers {
            return;
        }
        if self.form.footers.len() >= MAX_FOOTERS {
            self.input_error = Some("Too many footers");
            return;
        }
        self.mode = Mode::FooterNamePicker(FooterNamePickerState {
            query: Input::default(),
            highlighted: 0,
        });
    }

    fn handle_footer_name_picker(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Escape => {
                self.mode = Mode::Form;
                self.input_error = None;
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

    fn open_footer_editor(&mut self, index: Option<usize>) {
        if !self.sections.footers {
            return;
        }
        if index.is_none() && self.form.footers.len() >= MAX_FOOTERS {
            self.input_error = Some("Too many footers");
            return;
        }
        let footer = index.and_then(|index| self.form.footers.get(index));
        self.mode = Mode::FooterEditor(FooterEditorState {
            index,
            token: Input::new(footer.map_or_else(String::new, |footer| footer.token.clone())),
            value: Input::new(footer.map_or_else(String::new, |footer| footer.value.clone())),
            focus: FooterEditorFocus::Name,
        });
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
                editor.token = Input::new("BREAKING CHANGE".to_owned());
                editor.focus = FooterEditorFocus::Value;
            }
        }
    }

    fn handle_footer_editor(&mut self, event: AppEvent) -> AppAction {
        match event {
            AppEvent::Cancel => return AppAction::Cancel,
            AppEvent::Escape => self.mode = Mode::Form,
            AppEvent::Tab => self.move_footer_editor_focus(1),
            AppEvent::BackTab => self.move_footer_editor_focus(-1),
            AppEvent::Enter => {
                let insert_newline = matches!(&self.mode, Mode::FooterEditor(editor) if editor.focus == FooterEditorFocus::Value);
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
        editor.focus = match (editor.focus, direction.is_negative()) {
            (FooterEditorFocus::Name, false) | (FooterEditorFocus::Save, true) => {
                FooterEditorFocus::Value
            }
            (FooterEditorFocus::Value, false) => FooterEditorFocus::Save,
            (FooterEditorFocus::Value, true) => FooterEditorFocus::Name,
            (FooterEditorFocus::Save, false) => FooterEditorFocus::Name,
            (FooterEditorFocus::Name, true) => FooterEditorFocus::Save,
        };
    }

    fn save_footer_editor(&mut self) {
        let Mode::FooterEditor(editor) = &self.mode else {
            return;
        };
        let index = editor.index;
        let footer = Footer::new(editor.token.to_string(), editor.value.to_string());
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
        let Mode::FooterEditor(editor) = &mut self.mode else {
            return;
        };
        let (input, limit) = match editor.focus {
            FooterEditorFocus::Name => (&mut editor.token, MAX_FOOTER_TOKEN_LENGTH),
            FooterEditorFocus::Value => (&mut editor.value, MAX_FOOTER_VALUE_LENGTH),
            FooterEditorFocus::Save => return,
        };
        if let Edit::Insert(character) = edit {
            if character.is_control() && character != '\n' {
                self.input_error = Some("Control characters are not supported");
                return;
            }
            if input.to_string().chars().count() >= limit {
                self.input_error = Some("Field is too long");
                return;
            }
        }
        input.handle(edit.request());
        self.clear_validation_error();
    }

    fn paste_footer_editor(&mut self, value: &str) {
        let Ok(value) = sanitize_multiline_paste(value) else {
            self.input_error =
                Some("Paste contains unsupported control characters or is too large");
            return;
        };
        let Mode::FooterEditor(editor) = &self.mode else {
            return;
        };
        let (input, limit) = match editor.focus {
            FooterEditorFocus::Name => (&editor.token, MAX_FOOTER_TOKEN_LENGTH),
            FooterEditorFocus::Value => (&editor.value, MAX_FOOTER_VALUE_LENGTH),
            FooterEditorFocus::Save => return,
        };
        if input.to_string().chars().count() + value.chars().count() > limit {
            self.input_error = Some("Paste exceeds the field limit");
            return;
        }
        for character in value.chars() {
            self.edit_footer_editor(Edit::Insert(character));
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
        let choice = picker
            .choices_for(&self.message_policy)
            .get(picker.highlighted)
            .cloned();
        match choice {
            Some(TypeChoice::Standard(value)) | Some(TypeChoice::CustomQuery(value)) => {
                self.form.commit_type = Input::new(value);
                self.reset_preview_scroll();
            }
            None => {}
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
        if choices_len == 0 {
            return;
        }
        picker.highlighted =
            (picker.highlighted as isize + direction).rem_euclid(choices_len as isize) as usize;
    }

    fn select_scope_choice(&mut self) {
        let choice = match &self.mode {
            Mode::ScopePicker(picker) => {
                self.scope_choices(picker).get(picker.highlighted).cloned()
            }
            _ => return,
        };
        if let Some(TypeChoice::Standard(value) | TypeChoice::CustomQuery(value)) = choice {
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
        if len == 0 {
            return;
        }
        if let Mode::ScopePicker(picker) = &mut self.mode {
            picker.highlighted =
                (picker.highlighted as isize + direction).rem_euclid(len as isize) as usize;
        }
    }

    fn select_footer_name_choice(&mut self) {
        let Mode::FooterNamePicker(picker) = &self.mode else {
            return;
        };
        let choice = picker.choices().get(picker.highlighted).cloned();
        let Some(TypeChoice::Standard(name) | TypeChoice::CustomQuery(name)) = choice else {
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
        let len = picker.choices().len();
        if len > 0 {
            picker.highlighted =
                (picker.highlighted as isize + direction).rem_euclid(len as isize) as usize;
        }
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
                    self.input_error = Some("Control characters are not supported");
                    return;
                }
                if input.to_string().chars().count() >= field_limit(self.focus) {
                    self.input_error = Some("Field is too long");
                    return;
                }
            }
            input.handle(edit.request());
            self.reset_preview_scroll();
            self.clear_validation_error();
        }
    }

    fn paste_form(&mut self, value: &str) {
        if self.focus == Focus::Body {
            let Ok(value) = sanitize_multiline_paste(value) else {
                self.input_error =
                    Some("Paste contains unsupported control characters or is too large");
                return;
            };
            if self.form.body.to_string().chars().count() + value.chars().count() > MAX_BODY_LENGTH
            {
                self.input_error = Some("Paste exceeds the field limit");
                return;
            }
            for character in value.chars() {
                self.edit_form(Edit::Insert(character));
            }
            return;
        }
        let Ok(value) = sanitize_paste(value) else {
            self.input_error =
                Some("Paste contains unsupported control characters or is too large");
            return;
        };
        let existing_length = match self.focus {
            Focus::Scope => self.form.scope.to_string().chars().count(),
            Focus::Message => self.form.message.to_string().chars().count(),
            Focus::Body => self.form.body.to_string().chars().count(),
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

    fn edit_scope_picker(&mut self, edit: Edit) {
        let Mode::ScopePicker(picker) = &mut self.mode else {
            return;
        };
        if let Edit::Insert(character) = edit {
            if character.is_control() || matches!(character, '(' | ')') {
                self.input_error = Some("Scope contains an invalid character");
                return;
            }
            if picker.query.to_string().chars().count() >= MAX_SCOPE_LENGTH {
                self.input_error = Some("Field is too long");
                return;
            }
        }
        picker.query.handle(edit.request());
        picker.highlighted = 0;
        self.input_error = None;
    }

    fn paste_scope_picker(&mut self, value: &str) {
        let Ok(value) = sanitize_paste(value) else {
            self.input_error =
                Some("Paste contains unsupported control characters or is too large");
            return;
        };
        for character in value.chars() {
            self.edit_scope_picker(Edit::Insert(character));
        }
    }

    fn edit_footer_name_picker(&mut self, edit: Edit) {
        let Mode::FooterNamePicker(picker) = &mut self.mode else {
            return;
        };
        if let Edit::Insert(character) = edit {
            if character.is_control() {
                self.input_error = Some("Control characters are not supported");
                return;
            }
            if picker.query.to_string().chars().count() >= MAX_FOOTER_TOKEN_LENGTH {
                self.input_error = Some("Field is too long");
                return;
            }
        }
        picker.query.handle(edit.request());
        picker.highlighted = 0;
        self.input_error = None;
    }

    fn paste_footer_name_picker(&mut self, value: &str) {
        let Ok(value) = sanitize_paste(value) else {
            self.input_error =
                Some("Paste contains unsupported control characters or is too large");
            return;
        };
        let Mode::FooterNamePicker(picker) = &self.mode else {
            return;
        };
        if picker.query.to_string().chars().count() + value.chars().count()
            > MAX_FOOTER_TOKEN_LENGTH
        {
            self.input_error = Some("Paste exceeds the field limit");
            return;
        }
        for character in value.chars() {
            self.edit_footer_name_picker(Edit::Insert(character));
        }
    }

    fn submit(&mut self) -> AppAction {
        match self.draft() {
            Ok(_) => AppAction::Submit,
            Err(errors) => {
                let error = errors[0];
                if let DraftField::Footer(index) = error.field {
                    if !self.sections.footers {
                        self.focus = Focus::Message;
                        self.validation_error = Some(error);
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

    fn reset_preview_scroll(&mut self) {
        self.preview_scroll = 0;
    }

    fn scroll_preview(&mut self, direction: isize) {
        let limit = self.active_preview_scroll_limit();
        self.preview_scroll = if direction.is_negative() {
            self.preview_scroll.saturating_sub(1)
        } else {
            self.preview_scroll.saturating_add(1).min(limit)
        };
    }

    fn active_preview_scroll_limit(&self) -> u16 {
        match &self.mode {
            Mode::PreviewExpanded => self.expanded_preview_scroll_limit,
            Mode::Help(previous) if matches!(previous.as_ref(), Mode::PreviewExpanded) => {
                self.expanded_preview_scroll_limit
            }
            _ => self.preview_scroll_limit,
        }
    }

    fn move_staged(&mut self, direction: isize) {
        if self.staged_changes.files.is_empty() {
            self.focus = if direction < 0 {
                self.previous_focus()
            } else {
                self.next_focus()
            };
            return;
        }
        let last = self.staged_changes.files.len() - 1;
        if direction < 0 && self.staged_selected == 0 {
            self.focus = self.previous_focus();
        } else if direction > 0 && self.staged_selected == last {
            self.focus = self.next_focus();
        } else {
            self.staged_selected =
                (self.staged_selected as isize + direction).clamp(0, last as isize) as usize;
        }
    }
    fn next_focus(&self) -> Focus {
        let focuses = self.visible_focuses();
        let index = focuses
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        focuses[(index + 1) % focuses.len()]
    }

    fn previous_focus(&self) -> Focus {
        let focuses = self.visible_focuses();
        let index = focuses
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        focuses[(index + focuses.len() - 1) % focuses.len()]
    }

    fn ensure_visible_focus(&mut self) {
        if !self.visible_focuses().contains(&self.focus) {
            self.focus = Focus::CommitType;
            self.mode = Mode::Form;
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
        DraftField::Footer(_) => Focus::Message,
    }
}

fn field_limit(focus: Focus) -> usize {
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

fn sanitize_multiline_paste(value: &str) -> Result<String, ()> {
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

#[cfg(test)]
mod tests;
