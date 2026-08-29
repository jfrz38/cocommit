use anyhow::Result;

pub mod app;
pub mod commit;
pub mod config;
pub mod event;
pub mod git;
mod terminal;
mod ui;

fn main() -> Result<()> {
    git::preflight()?;
    let config = config::load()?;
    let mut app = app::App::new(config.sign);
    match terminal::run(&mut app)? {
        terminal::TerminalResult::Cancelled => {}
        terminal::TerminalResult::Submitted { draft, sign } => {
            git::commit(&draft.render_message(), sign)?;
        }
    }
    Ok(())
}
