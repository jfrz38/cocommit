//! Application state and state transitions.

use tui_input::{Input, InputRequest};

use crate::{
    commit::{CommitDraft, MessagePolicy, SubjectPolicy, ValidationError},
    settings::UiSections,
    staging::{StagedChanges, StagedFile},
};

mod footer_editor;
mod form;
mod input;
mod interaction;
mod navigation;
mod picker;
mod preview_state;

pub use footer_editor::{FooterEditorFocus, FooterEditorState};
pub use form::FormState;
pub use picker::{FooterNamePickerState, TypeChoice, TypePickerState};
pub use preview_state::PreviewState;
use staged_selection::{StagedSelection, ToggleResult};

mod staged_selection;

#[cfg(test)]
use crate::commit::Footer;

pub use crate::commit::policy::DEFAULT_TYPES as STANDARD_TYPES;
pub const MAX_TYPE_LENGTH: usize = 64;
pub const MAX_SCOPE_LENGTH: usize = 128;
pub const MAX_MESSAGE_LENGTH: usize = 512;
pub const MAX_ISSUE_LENGTH: usize = 20;
pub const MAX_BODY_LENGTH: usize = 4096;
pub const MAX_FOOTER_TOKEN_LENGTH: usize = 64;
pub const MAX_FOOTER_VALUE_LENGTH: usize = 2048;
pub const MAX_FOOTERS: usize = 32;
pub const MAX_PASTE_LENGTH: usize = 4096;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Continue,
    Cancel,
    Submit(SubmissionIntent),
}

/// A validated commit request ready for execution after terminal restoration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionIntent {
    pub draft: CommitDraft,
    pub sign: bool,
    pub excluded_files: Vec<StagedFile>,
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

#[derive(Debug, Clone)]
pub struct App {
    form: FormState,
    focus: Focus,
    mode: Mode,
    sign: bool,
    sections: UiSections,
    staged_selection: StagedSelection,
    footer_selected: usize,
    preview_state: PreviewState,
    message_policy: MessagePolicy,
    feedback: Feedback,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Feedback {
    pub(crate) validation_error: Option<ValidationError>,
    pub(crate) input_error: Option<InputError>,
    pub(crate) operation_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputError {
    ControlCharacter,
    FieldTooLong,
    InvalidScopeCharacter,
    TooManyFooters,
    PasteUnsupportedContent,
    PasteExceedsFieldLimit,
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
            staged_selection: StagedSelection::default(),
            footer_selected: 0,
            preview_state: PreviewState::default(),
            message_policy: MessagePolicy::default(),
            feedback: Feedback::default(),
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
        self.staged_selection.set_enabled(sections.staged_changes);
        self.ensure_visible_focus();
        self
    }

    pub fn form(&self) -> &FormState {
        &self.form
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn sign(&self) -> bool {
        self.sign
    }

    pub fn sections(&self) -> UiSections {
        self.sections
    }

    pub fn footer_selected(&self) -> usize {
        self.footer_selected
    }

    pub fn preview_state(&self) -> &PreviewState {
        &self.preview_state
    }

    pub(crate) fn feedback(&self) -> &Feedback {
        &self.feedback
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

    pub fn subject_policy(&self) -> &SubjectPolicy {
        &self.message_policy.subject
    }

    pub fn has_scope_suggestions(&self) -> bool {
        !self.message_policy.scope_suggestions.is_empty()
    }

    pub fn types_are_restricted(&self) -> bool {
        self.message_policy.types_are_restricted
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
        self.staged_selection
            .replace(staged_changes, self.sections.staged_changes);
    }

    pub fn staged_changes(&self) -> &StagedChanges {
        self.staged_selection.changes()
    }

    pub fn staged_selected(&self) -> usize {
        self.staged_selection.selected()
    }

    /// Synchronizes preview scroll limits calculated from the current terminal viewport.
    pub fn set_preview_scroll_limits(&mut self, form_limit: u16, expanded_limit: Option<u16>) {
        self.preview_state
            .set_limits(form_limit, expanded_limit, self.preview_is_expanded());
    }

    pub fn staged_file_is_included(&self, index: usize) -> bool {
        self.staged_selection.is_included(index)
    }

    pub fn included_staged_count(&self) -> usize {
        self.staged_selection.included_count()
    }

    pub fn excluded_staged_files(&self) -> Vec<StagedFile> {
        self.staged_selection.excluded_files()
    }

    fn set_operation_error(&mut self, message: impl Into<String>) {
        self.feedback.operation_status = Some(message.into());
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
        self.form
            .project(self.sections, issue)
            .render_message_with_policy(&self.message_policy)
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

#[cfg(test)]
mod tests;
