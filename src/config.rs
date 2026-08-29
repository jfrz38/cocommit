//! Global application configuration.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Global preferences for cocommit's user interface.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub sign: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { sign: true }
    }
}

/// Returns the optional global configuration file location.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|directory| directory.join("cocommit").join("config.toml"))
}

/// Loads the global configuration, or defaults when no configuration is available.
pub fn load() -> Result<Config> {
    config_path().map_or_else(|| Ok(Config::default()), |path| load_from_path(&path))
}

fn load_from_path(path: &Path) -> Result<Config> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Config::default()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!("failed to read configuration file at {}", path.display())
            });
        }
    };

    parse(&contents)
        .with_context(|| format!("failed to parse configuration file at {}", path.display()))
}

fn parse(contents: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(contents)
}

#[cfg(test)]
mod tests;
