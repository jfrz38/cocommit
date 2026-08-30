//! Command-line parsing and public usage text.

use std::ffi::OsString;

/// The only supported non-interactive commands.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run,
    Help,
    Version,
}

/// A usage error that must not initialize the interactive terminal.
#[derive(Debug, PartialEq, Eq)]
pub struct UsageError {
    argument: OsString,
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unrecognized argument: {}",
            self.argument.to_string_lossy()
        )
    }
}

impl std::error::Error for UsageError {}

/// Parses the small stable command-line contract without a parser dependency.
pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let mut arguments = arguments.into_iter();
    let Some(argument) = arguments.next() else {
        return Ok(Command::Run);
    };

    if arguments.next().is_some() {
        return Err(UsageError { argument });
    }

    match argument.to_str() {
        Some("-h" | "--help") => Ok(Command::Help),
        Some("-V" | "--version") => Ok(Command::Version),
        _ => Err(UsageError { argument }),
    }
}

pub const USAGE: &str =
    "Usage: cocommit\n\nOptions:\n  -h, --help     Print help\n  -V, --version  Print version";

#[cfg(test)]
mod tests;
