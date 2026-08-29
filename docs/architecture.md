# Architecture

## Design

The application is a synchronous Rust binary. Its behavior is small enough that async runtimes, dependency injection, generic repositories, and domain layers would add cost without solving a concrete problem.

The Conventional Commit model is pure Rust and independent of Ratatui. UI code owns editing and rendering; Git code owns explicit Git commands.

## Proposed structure

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
Makefile
.github/
  workflows/
    ci.yml
src/
  lib.rs
  main.rs
  app.rs
  commit.rs
  config.rs
  event.rs
  git.rs
  terminal.rs
  ui.rs
tests/
  git_integration.rs
```

`lib.rs` exposes the production modules to the binary and black-box integration tests. Unit tests can live beside their modules. The integration test is separate because it executes a real temporary Git repository.

## Development tooling

`rust-toolchain.toml` fixes development and CI to Rust 1.94.1 with the `clippy` and `rustfmt` components. `Makefile` is the local quality interface: `make check` runs formatting, linting, tests, and a build against `Cargo.lock`.

`.github/workflows/ci.yml` runs that quality suite on Ubuntu. It also checks compilation on Windows and macOS for pull requests to `main`, scheduled runs, and manual dispatches, without requiring GNU Make on those runners.

## Module responsibilities

| Module | Responsibility |
|---|---|
| `main.rs` | Orchestrates preflight, config loading, terminal lifecycle, TUI result, and final Git execution. |
| `commit.rs` | Defines the commit draft, canonical rendering, and validation. Has no terminal or Git dependency. |
| `config.rs` | Defines defaults, resolves the global path, reads and parses TOML. |
| `git.rs` | Runs explicit Git commands, interprets exit statuses, and constructs `git commit` arguments. |
| `app.rs` | Holds editable form state, focus, popup state, validation feedback, and state transitions. |
| `event.rs` | Maps Crossterm events to small application actions. |
| `ui.rs` | Renders the form, preview, type picker, status, and footer from `App`. |
| `terminal.rs` | Owns raw mode, alternate screen, cursor restoration, and the synchronous event loop boundary. |

Dependencies point inward:

```text
main -> config, git, terminal, app, event, ui
app -> commit
event -> app
ui -> app, commit
git -> std::process
```

## Core domain model

```rust
pub struct CommitDraft {
    pub commit_type: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub message: String,
    pub issue: Option<u64>,
}
```

Signing is deliberately not part of `CommitDraft`: it changes the Git command, not the Conventional Commit message.

```rust
impl CommitDraft {
    pub fn render_message(&self) -> String;
    pub fn validate(&self) -> Result<(), Vec<ValidationError>>;
    pub fn validated_message(&self) -> Result<String, Vec<ValidationError>>;
}
```

`render_message` is the sole message formatter. `validated_message` validates first and then delegates to it, preventing duplicate formatting logic.

`ValidationError` should identify its relevant form field so the UI can focus it after a failed submission.

## Application state

```rust
pub struct App {
    pub form: FormState,
    pub focus: Focus,
    pub mode: Mode,
    pub sign: bool,
    pub validation_error: Option<ValidationError>,
}

pub enum Focus {
    CommitType,
    Scope,
    Breaking,
    Message,
    Issue,
    Sign,
    Submit,
}

pub enum Mode {
    Form,
    TypePicker(TypePickerState),
}

pub enum AppAction {
    Continue,
    Cancel,
    Submit,
}
```

`FormState` contains the editable text widgets and converts them into `CommitDraft`. It may use `tui-input` to preserve Unicode-aware cursor behavior without leaking UI concerns into the domain model.

## Main flow

```text
preflight Git -> load config -> initialize terminal -> run UI loop
    -> cancel: restore terminal and exit successfully
    -> submit: validate -> restore terminal -> git commit -> exit with Git status
```

The terminal is restored before invoking Git. This is essential for hooks, signing prompts, pinentry, and normal Git output.
