//! Terminal lifecycle management.

use std::io;

use anyhow::{Context, Result, anyhow};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, EnableBracketedPaste, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::{App, AppAction},
    commit::CommitDraft,
    event, ui,
};

/// The result returned by the interactive form after terminal restoration.
pub enum TerminalResult {
    Cancelled,
    Submitted { draft: CommitDraft, sign: bool },
}

/// Owns the terminal modes that must be restored before returning to Git.
pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    raw_mode: bool,
    alternate_screen: bool,
    bracketed_paste: bool,
    cursor_hidden: bool,
}

impl TerminalSession {
    /// Initializes Ratatui after placing the terminal in its interactive mode.
    pub fn new() -> Result<Self> {
        enable_raw_mode().context("failed to enable terminal raw mode")?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableBracketedPaste, Hide) {
            let _ = execute!(stdout, Show, DisableBracketedPaste, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(error).context("failed to initialize terminal screen");
        }

        let backend = CrosstermBackend::new(stdout);
        match Terminal::new(backend) {
            Ok(terminal) => Ok(Self {
                terminal,
                raw_mode: true,
                alternate_screen: true,
                bracketed_paste: true,
                cursor_hidden: true,
            }),
            Err(error) => {
                let _ = execute!(
                    io::stdout(),
                    Show,
                    DisableBracketedPaste,
                    LeaveAlternateScreen
                );
                let _ = disable_raw_mode();
                Err(error).context("failed to create terminal renderer")
            }
        }
    }

    fn draw(&mut self, app: &App) -> Result<()> {
        self.terminal
            .draw(|frame| ui::render(frame, app))
            .context("failed to draw terminal UI")?;
        Ok(())
    }

    /// Runs the synchronous input loop until the user cancels or submits a valid draft.
    pub fn run(&mut self, app: &mut App) -> Result<TerminalResult> {
        loop {
            self.draw(app)?;
            let event = read().context("failed to read terminal event")?;
            let Some(event) = event::map(event) else {
                continue;
            };

            match app.handle(event) {
                AppAction::Continue => {}
                AppAction::Cancel => return Ok(TerminalResult::Cancelled),
                AppAction::Submit => {
                    let draft = app
                        .draft()
                        .map_err(|_| anyhow!("submitted an invalid draft"))?;
                    return Ok(TerminalResult::Submitted {
                        draft,
                        sign: app.sign,
                    });
                }
            }
        }
    }

    /// Attempts every cleanup operation, retaining the first failure for the caller.
    pub fn restore(&mut self) -> Result<()> {
        let mut failure = None;
        let backend = self.terminal.backend_mut();

        if self.cursor_hidden {
            if let Err(error) = execute!(backend, Show) {
                failure.get_or_insert(error);
            }
            self.cursor_hidden = false;
        }
        if self.bracketed_paste {
            if let Err(error) = execute!(backend, DisableBracketedPaste) {
                failure.get_or_insert(error);
            }
            self.bracketed_paste = false;
        }
        if self.alternate_screen {
            if let Err(error) = execute!(backend, LeaveAlternateScreen) {
                failure.get_or_insert(error);
            }
            self.alternate_screen = false;
        }
        if self.raw_mode {
            if let Err(error) = disable_raw_mode() {
                failure.get_or_insert(error);
            }
            self.raw_mode = false;
        }

        failure.map_or(Ok(()), |error| {
            Err(error).context("failed to restore terminal")
        })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Runs the UI and guarantees a restoration attempt before returning its result.
pub fn run(app: &mut App) -> Result<TerminalResult> {
    let mut session = TerminalSession::new()?;
    let result = session.run(app);
    let restore_result = session.restore();

    match (result, restore_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(restore_error)) => Err(error.context(format!(
            "terminal restoration also failed: {restore_error:#}"
        ))),
    }
}
