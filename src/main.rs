use std::env;

use anyhow::{Context, Result};
use cocommit::{app, config, git, terminal};

fn main() -> Result<()> {
    let working_directory = env::current_dir().context("failed to determine current directory")?;
    git::preflight(&working_directory)?;
    let config = config::load()?;
    let mut app = app::App::new(config.sign);
    match terminal::run(&mut app)? {
        terminal::TerminalResult::Cancelled => {}
        terminal::TerminalResult::Submitted { draft, sign } => {
            git::commit(&working_directory, &draft.render_message(), sign)?;
        }
    }
    Ok(())
}
