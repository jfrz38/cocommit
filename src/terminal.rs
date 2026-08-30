//! Terminal lifecycle management.

use std::{
    io::{self, IsTerminal},
    panic,
    sync::{
        Once,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result, anyhow, bail};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, EnableBracketedPaste, read},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

#[cfg(unix)]
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
#[cfg(unix)]
use std::sync::OnceLock;

use crate::{
    app::{App, AppAction},
    commit::CommitDraft,
    event, ui,
};

/// The result returned by the interactive form after terminal restoration.
pub enum TerminalResult {
    Cancelled,
    Submitted {
        draft: CommitDraft,
        sign: bool,
        excluded_files: Vec<crate::git::StagedFile>,
    },
}

static TERMINAL_STATE_ACTIVE: AtomicBool = AtomicBool::new(false);
static PANIC_HOOK: Once = Once::new();

#[cfg(unix)]
static SIGNAL_HANDLERS: OnceLock<Result<(), String>> = OnceLock::new();

/// Rejects redirected standard streams before changing terminal state.
pub fn ensure_standard_streams_are_interactive() -> Result<()> {
    ensure_interactive(io::stdin().is_terminal(), io::stdout().is_terminal())
}

fn ensure_interactive(stdin_is_terminal: bool, stdout_is_terminal: bool) -> Result<()> {
    match (stdin_is_terminal, stdout_is_terminal) {
        (true, true) => Ok(()),
        (false, false) => bail!("standard input and standard output must be interactive terminals"),
        (false, true) => bail!("standard input must be an interactive terminal"),
        (true, false) => bail!("standard output must be an interactive terminal"),
    }
}

/// Owns the terminal modes that must be restored before returning to Git.
pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    /// Initializes Ratatui after placing the terminal in its interactive mode.
    pub fn new() -> Result<Self> {
        install_restoration_handlers()?;
        // Mark recovery active first so a signal during raw-mode activation is still restored.
        TERMINAL_STATE_ACTIVE.store(true, Ordering::Release);
        if let Err(error) = enable_raw_mode() {
            let _ = restore_terminal_state();
            return Err(error).context("failed to enable terminal raw mode");
        }
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableBracketedPaste, Hide) {
            let _ = restore_terminal_state();
            return Err(error).context("failed to initialize terminal screen");
        }

        let backend = CrosstermBackend::new(stdout);
        match Terminal::new(backend) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let _ = restore_terminal_state();
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
                        excluded_files: app.excluded_staged_files(),
                    });
                }
            }
        }
    }

    /// Attempts every cleanup operation, retaining the first failure for the caller.
    pub fn restore(&mut self) -> Result<()> {
        restore_terminal_state()
    }
}

/// Restores every terminal mode once, including after partial initialization.
fn restore_terminal_state() -> Result<()> {
    if !TERMINAL_STATE_ACTIVE.swap(false, Ordering::AcqRel) {
        return Ok(());
    }

    let mut failure = None;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, Show) {
        failure.get_or_insert(error);
    }
    if let Err(error) = execute!(stdout, DisableBracketedPaste) {
        failure.get_or_insert(error);
    }
    if let Err(error) = execute!(stdout, LeaveAlternateScreen) {
        failure.get_or_insert(error);
    }
    if let Err(error) = disable_raw_mode() {
        failure.get_or_insert(error);
    }

    failure.map_or(Ok(()), |error| {
        Err(error).context("failed to restore terminal")
    })
}

fn install_restoration_handlers() -> Result<()> {
    install_panic_hook();
    #[cfg(unix)]
    install_unix_signal_handlers()?;
    Ok(())
}

fn install_panic_hook() {
    PANIC_HOOK.call_once(|| {
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = restore_terminal_state();
            previous_hook(panic_info);
        }));
    });
}

#[cfg(unix)]
fn install_unix_signal_handlers() -> Result<()> {
    let installation = SIGNAL_HANDLERS.get_or_init(|| {
        let mut signals =
            Signals::new([SIGHUP, SIGINT, SIGQUIT, SIGTERM]).map_err(|error| error.to_string())?;
        std::thread::Builder::new()
            .name("cocommit-terminal-signals".to_owned())
            .spawn(move || {
                if let Some(signal) = signals.forever().next() {
                    let _ = restore_terminal_state();
                    std::process::exit(128 + signal);
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    });
    installation
        .as_ref()
        .map_err(|error| anyhow!(error.clone()))
        .copied()
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

#[cfg(test)]
mod tests;
