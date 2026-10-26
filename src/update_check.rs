//! Best-effort detection of releases newer than the running version.
//!
//! Every problem — a missing cache location, an unreachable network, a
//! timeout, or unexpected output — reports `None` so the terminal interface
//! always opens as usual.

use std::{
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Repository whose `vX.Y.Z` tags describe published versions.
const RELEASES_REPOSITORY: &str = "https://github.com/jfrz38/cocommit.git";
/// Deadline applied to a single network check.
const CHECK_TIMEOUT: Duration = Duration::from_secs(2);
/// Freshness window after a successful check.
const SUCCESS_TTL: Duration = Duration::from_secs(24 * 60 * 60);
/// Freshness window after a failed check, keeping offline starts cheap.
const FAILURE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
/// Upper bound on captured `git ls-remote` output.
const MAX_OUTPUT_BYTES: u64 = 64 * 1024;
/// Environment variable that disables the check regardless of configuration.
const DISABLE_VARIABLE: &str = "COCOMMIT_UPDATE_CHECK";

/// A published version newer than the running one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateNotice {
    /// Normalized `X.Y.Z` of the newest published release.
    pub latest: String,
}

/// Returns `true` unless configuration or the environment disables checking.
pub fn enabled(configured: bool) -> bool {
    configured && !disables(std::env::var_os(DISABLE_VARIABLE).as_deref())
}

/// Returns a notice when a newer release exists, reporting `None` otherwise.
pub fn check(current: &str) -> Option<UpdateNotice> {
    let cache = cache_path();
    check_with(
        current,
        cache.as_deref(),
        SystemTime::now(),
        fetch_latest_tag,
    )
}

/// Starts a background check so the terminal interface never waits for it.
///
/// The channel stays open until the outcome is known: a newer release sends
/// its notice, while every other outcome only closes the channel.
pub fn start(current: &str) -> Option<Receiver<UpdateNotice>> {
    let current = current.to_owned();
    spawn_check(move || check(&current))
}

/// Runs `fetch` on a named thread and forwards its notice when one exists.
fn spawn_check(
    fetch: impl FnOnce() -> Option<UpdateNotice> + Send + 'static,
) -> Option<Receiver<UpdateNotice>> {
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("cocommit-update-check".to_owned())
        .spawn(move || {
            if let Some(notice) = fetch() {
                let _ = sender.send(notice);
            }
        })
        .ok()?;
    Some(receiver)
}

fn check_with(
    current: &str,
    cache: Option<&Path>,
    now: SystemTime,
    fetch: impl FnOnce() -> Option<String>,
) -> Option<UpdateNotice> {
    let current_version = parse_version(current)?;
    if let Some(cached) = cache.and_then(|path| fresh_cache(path, now)) {
        return cached
            .latest
            .as_deref()
            .and_then(|latest| newer_notice(latest, &current_version));
    }
    let latest = fetch();
    if let Some(path) = cache {
        write_cache(path, latest.as_deref(), now);
    }
    latest
        .as_deref()
        .and_then(|latest| newer_notice(latest, &current_version))
}

/// Builds a notice only when `latest` is strictly newer than `current`.
fn newer_notice(latest: &str, current: &Version) -> Option<UpdateNotice> {
    let latest = parse_version(latest)?;
    (latest > *current).then(|| {
        let (major, minor, patch) = latest;
        UpdateNotice {
            latest: format!("{major}.{minor}.{patch}"),
        }
    })
}

/// Parses a plain `X.Y.Z` release version, tolerating a leading `v`.
fn parse_version(value: &str) -> Option<Version> {
    let trimmed = value.trim();
    let value = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let mut numbers = value.split('.');
    let version = (
        parse_number(numbers.next()?)?,
        parse_number(numbers.next()?)?,
        parse_number(numbers.next()?)?,
    );
    numbers.next().is_none().then_some(version)
}

fn parse_number(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

/// Returns the highest `vX.Y.Z` tag found in `git ls-remote` output.
fn newest_tag(output: &str) -> Option<String> {
    output
        .lines()
        .filter_map(|line| {
            let reference = line.split_whitespace().nth(1)?;
            let tag = reference.strip_prefix("refs/tags/v")?;
            parse_version(tag)
        })
        .max()
        .map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
}

/// Fetches the newest published tag, reporting `None` on any failure.
fn fetch_latest_tag() -> Option<String> {
    list_tags(RELEASES_REPOSITORY)
        .ok()
        .and_then(|tags| newest_tag(&tags))
}

/// Reads version tags from a repository within the check deadline.
fn list_tags(repository: &str) -> Result<String> {
    list_tags_with_timeout(repository, CHECK_TIMEOUT)
}

fn list_tags_with_timeout(repository: &str, timeout: Duration) -> Result<String> {
    let mut child = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", repository, "refs/tags/v*"])
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("git ls-remote could not start")?;
    let stdout = child
        .stdout
        .take()
        .context("git ls-remote has no stdout pipe")?;
    let reader = read_bounded(stdout);
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .context("git ls-remote status could not be read")?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("git ls-remote exceeded {timeout:?}");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = receive_output(reader, deadline)?;
    if !status.success() {
        bail!("git ls-remote exited with {status}");
    }
    String::from_utf8(output).context("git ls-remote output is not valid UTF-8")
}

/// Streams the pipe into a channel so it cannot block the deadline.
fn read_bounded(reader: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut output = Vec::new();
        let result = reader
            .take(MAX_OUTPUT_BYTES + 1)
            .read_to_end(&mut output)
            .map(|_| output);
        let _ = sender.send(result);
    });
    receiver
}

fn receive_output(receiver: Receiver<io::Result<Vec<u8>>>, deadline: Instant) -> Result<Vec<u8>> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    let output = match receiver.recv_timeout(remaining) {
        Ok(output) => output.context("git ls-remote output could not be read")?,
        Err(_) => bail!("git ls-remote output arrived too late"),
    };
    if output.len() as u64 > MAX_OUTPUT_BYTES {
        bail!("git ls-remote output exceeds {MAX_OUTPUT_BYTES} bytes");
    }
    Ok(output)
}

/// Interprets the opt-out environment variable value.
fn disables(value: Option<&OsStr>) -> bool {
    value.and_then(OsStr::to_str).is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        )
    })
}

/// Returns the cached outcome while it is still inside its freshness window.
fn fresh_cache(path: &Path, now: SystemTime) -> Option<CachedCheck> {
    let cached: CachedCheck = toml::from_str(&fs::read_to_string(path).ok()?).ok()?;
    let checked_at = UNIX_EPOCH.checked_add(Duration::from_secs(cached.checked_at))?;
    let age = now.duration_since(checked_at).ok()?;
    let ttl = if cached.latest.is_some() {
        SUCCESS_TTL
    } else {
        FAILURE_TTL
    };
    (age < ttl).then_some(cached)
}

/// Persists a check outcome, ignoring every storage problem.
fn write_cache(path: &Path, latest: Option<&str>, now: SystemTime) {
    let Ok(age) = now.duration_since(UNIX_EPOCH) else {
        return;
    };
    let record = CacheRecord {
        checked_at: age.as_secs(),
        latest,
    };
    let Ok(contents) = toml::to_string(&record) else {
        return;
    };
    if let Some(directory) = path.parent()
        && fs::create_dir_all(directory).is_err()
    {
        return;
    }
    let _ = fs::write(path, contents);
}

/// Returns the optional cache file location.
fn cache_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|directory| directory.join("cocommit").join("update-check.toml"))
}

/// A stored check outcome after its freshness window has been applied.
#[derive(Debug, Deserialize)]
struct CachedCheck {
    checked_at: u64,
    #[serde(default)]
    latest: Option<String>,
}

/// The check outcome written to the cache file.
#[derive(Serialize)]
struct CacheRecord<'a> {
    checked_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest: Option<&'a str>,
}

/// Comparable `X.Y.Z` release version.
type Version = (u64, u64, u64);

#[cfg(test)]
mod tests;
