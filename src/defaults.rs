//! Resolution of trusted, global-only commands that prefill composer fields.

use std::{
    io::{self, Read},
    path::Path,
    process::{Command, ExitStatus, Stdio},
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail};

use crate::{
    app::{MAX_BODY_LENGTH, MAX_ISSUE_LENGTH, MAX_SCOPE_LENGTH, MAX_TYPE_LENGTH},
    commit::{CommitDraft, DraftField, MessagePolicy},
    config::{ComposerDefaultCommands, DefaultCommand},
    settings::{ComposerDefaults, UiSections},
};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_OUTPUT_BYTES: u64 = 8 * 1024;

/// Runs configured commands for visible fields and validates their one-line values.
pub fn resolve(
    commands: &ComposerDefaultCommands,
    work_tree_root: &Path,
    sections: UiSections,
    policy: &MessagePolicy,
) -> Result<ComposerDefaults> {
    Ok(ComposerDefaults {
        commit_type: resolve_field(
            "type",
            sections.commit_type,
            commands.commit_type.as_ref(),
            work_tree_root,
            MAX_TYPE_LENGTH,
            |value| validate_type(value, policy),
        )?,
        scope: resolve_field(
            "scope",
            sections.commit_type && sections.scope,
            commands.scope.as_ref(),
            work_tree_root,
            MAX_SCOPE_LENGTH,
            validate_scope,
        )?,
        body: resolve_field(
            "body",
            sections.body,
            commands.body.as_ref(),
            work_tree_root,
            MAX_BODY_LENGTH,
            |_| Ok(()),
        )?,
        issue: resolve_field(
            "issue",
            sections.issue,
            commands.issue.as_ref(),
            work_tree_root,
            MAX_ISSUE_LENGTH,
            |value| {
                CommitDraft::parse_issue(value)
                    .map(|_| ())
                    .map_err(|_| anyhow!("must be a decimal number"))
            },
        )?,
    })
}

fn resolve_field(
    field: &str,
    visible: bool,
    command: Option<&DefaultCommand>,
    work_tree_root: &Path,
    max_length: usize,
    validate: impl FnOnce(&str) -> Result<()>,
) -> Result<Option<String>> {
    if !visible {
        return Ok(None);
    }
    let Some(command) = command else {
        return Ok(None);
    };
    let value = run(command, work_tree_root)
        .with_context(|| format!("failed to resolve default for {field}"))?;
    if value.is_empty() {
        bail!("default for {field} cannot be empty");
    }
    if value.chars().count() > max_length {
        bail!("default for {field} exceeds its {max_length}-character limit");
    }
    if value.chars().any(char::is_control) {
        bail!("default for {field} contains a control character or multiple lines");
    }
    validate(&value).with_context(|| format!("invalid default for {field}"))?;
    Ok(Some(value))
}

fn run(spec: &DefaultCommand, work_tree_root: &Path) -> Result<String> {
    let Some(executable) = spec
        .command
        .first()
        .filter(|value| !value.trim().is_empty())
    else {
        bail!("command must contain an executable");
    };
    let mut child = Command::new(executable)
        .args(&spec.command[1..])
        .current_dir(work_tree_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("could not start `{executable}`"))?;
    let stdout = read_bounded(child.stdout.take().expect("stdout is piped"));
    let stderr = read_bounded(child.stderr.take().expect("stderr is piped"));

    let deadline = Instant::now() + COMMAND_TIMEOUT;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .context("could not inspect command status")?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(timeout_error());
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout = receive_output(stdout, deadline)?;
    let stderr = receive_output(stderr, deadline)?;
    ensure_success(status, &stderr)?;
    if stdout.len() > MAX_OUTPUT_BYTES as usize {
        bail!("command output exceeds {MAX_OUTPUT_BYTES} bytes");
    }
    let stdout = String::from_utf8(stdout).context("command output is not valid UTF-8")?;
    let value = stdout.trim().to_owned();
    if value.contains(['\n', '\r']) {
        bail!("command must output exactly one line");
    }
    Ok(value)
}

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
    match receiver.recv_timeout(remaining) {
        Ok(result) => result.context("could not read command output"),
        Err(RecvTimeoutError::Timeout) => Err(timeout_error()),
        Err(RecvTimeoutError::Disconnected) => bail!("command output reader stopped unexpectedly"),
    }
}

fn timeout_error() -> anyhow::Error {
    anyhow!(
        "command timed out after {} seconds",
        COMMAND_TIMEOUT.as_secs()
    )
}

fn ensure_success(status: ExitStatus, stderr: &[u8]) -> Result<()> {
    if status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(stderr).trim().to_owned();
    if stderr.is_empty() {
        bail!("command failed with status {status}");
    }
    bail!("command failed with status {status}: {stderr}")
}

fn validate_type(value: &str, policy: &MessagePolicy) -> Result<()> {
    let draft = CommitDraft::new(
        Some(value.to_owned()),
        None,
        false,
        "valid subject".to_owned(),
        None,
        None,
        Vec::new(),
    );
    if draft.validate_with_policy(policy).is_err_and(|errors| {
        errors
            .iter()
            .any(|error| error.field == DraftField::CommitType)
    }) {
        bail!("value is not an allowed commit type");
    }
    Ok(())
}

fn validate_scope(value: &str) -> Result<()> {
    if value.contains(['\n', '\r', '(', ')']) {
        bail!("value contains a forbidden scope character");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_fields_do_not_execute_their_commands() {
        let commands = ComposerDefaultCommands {
            commit_type: Some(DefaultCommand {
                command: vec!["executable-that-does-not-exist-cocommit".to_owned()],
            }),
            ..ComposerDefaultCommands::default()
        };
        let sections = UiSections {
            commit_type: false,
            ..UiSections::default()
        };

        assert_eq!(
            resolve(
                &commands,
                Path::new("."),
                sections,
                &MessagePolicy::default()
            )
            .unwrap(),
            ComposerDefaults::default()
        );
    }

    #[test]
    fn rejects_values_that_do_not_match_the_field_contract() {
        assert!(validate_type("bad type", &MessagePolicy::default()).is_err());
        assert!(validate_scope("bad(scope").is_err());
    }

    #[test]
    fn executes_a_program_without_shell_parsing() {
        let value = run(
            &DefaultCommand {
                command: vec!["rustc".to_owned(), "--version".to_owned()],
            },
            Path::new("."),
        )
        .expect("rustc should be available to the test suite");

        assert!(value.starts_with("rustc "));
    }

    #[test]
    fn rejects_a_programmatic_command_without_an_executable() {
        let error = run(
            &DefaultCommand {
                command: Vec::new(),
            },
            Path::new("."),
        )
        .unwrap_err();

        assert!(error.to_string().contains("must contain an executable"));
    }

    #[test]
    fn output_capture_observes_the_command_deadline() {
        let (_sender, receiver) = mpsc::channel();
        let deadline = Instant::now() + Duration::from_millis(1);

        assert!(
            receive_output(receiver, deadline)
                .unwrap_err()
                .to_string()
                .contains("timed out")
        );
    }
}
