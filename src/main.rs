use std::{env, process::ExitCode};

use anyhow::{Context, Result};
use cocommit::{app, cli, config, git, terminal};

const EXIT_FAILURE: u8 = 1;
const EXIT_USAGE: u8 = 2;

fn main() -> ExitCode {
    match cli::parse(env::args_os().skip(1)) {
        Ok(cli::Command::Help) => {
            println!("{}", cli::USAGE);
            ExitCode::SUCCESS
        }
        Ok(cli::Command::Version) => {
            println!("cocommit {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(cli::Command::Run) => match run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("cocommit: {error:#}");
                ExitCode::from(EXIT_FAILURE)
            }
        },
        Err(error) => {
            eprintln!("cocommit: {error}\n\n{}", cli::USAGE);
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn run() -> Result<()> {
    terminal::ensure_standard_streams_are_interactive()?;
    let working_directory = env::current_dir().context("failed to determine current directory")?;
    let staged_changes = git::preflight(&working_directory)?;
    let work_tree_root = git::work_tree_root(&working_directory)?;
    let config = config::load(&work_tree_root)?;
    let mut app = app::App::new(config.ui.sign);
    app.set_staged_changes(staged_changes);
    match terminal::run(&mut app)? {
        terminal::TerminalResult::Cancelled => {}
        terminal::TerminalResult::Submitted {
            draft,
            sign,
            excluded_files,
        } => {
            if !excluded_files.is_empty() {
                git::unstage(&working_directory, &excluded_files)?;
            }
            git::commit(&working_directory, &draft.render_message(), sign)?;
        }
    }
    Ok(())
}
