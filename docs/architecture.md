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
   cli.rs
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
| `main.rs` | Orchestrates preflight, config loading, terminal lifecycle, TUI results, index operations, and final Git execution. |
| `cli.rs` | Parses the small command-line contract and provides usage text without terminal or Git dependencies. |
| `commit.rs` | Defines the commit draft, canonical rendering, and validation. Has no terminal or Git dependency. |
| `config.rs` | Defines layered UI preferences and message policy, resolves global and repository paths, parses versioned TOML, and merges configuration. |
| `git.rs` | Runs explicit Git commands, interprets exit statuses, constructs `git commit` and literal unstage arguments, and reads the staged index. |
| `app.rs` | Holds editable form state, staged-file inclusion choices, focus, popup state, feedback, and pure state transitions. |
| `event.rs` | Maps Crossterm events to small application actions. |
| `ui.rs` | Renders the form, preview, type picker, status, and footer from `App`. |
| `terminal.rs` | Owns raw mode, alternate screen, cursor restoration, panic and Unix-signal cleanup, and the synchronous event loop boundary. |

Dependencies point inward:

```text
main -> cli, config, git, terminal, app, event, ui
app -> commit, config
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
parse CLI -> verify interactive streams -> preflight Git -> load config -> initialize terminal -> run UI loop
    -> cancel: restore terminal and exit successfully
    -> submit with exclusions: restore terminal -> Git unstage excluded files -> Git commit
    -> submit: validate -> restore terminal -> git commit -> exit with Git status
```

The terminal is restored before invoking Git. This is essential for hooks, signing prompts, pinentry, and normal Git output.

Before loading repository configuration, `main` asks Git for the work-tree root. `config` merges built-in defaults, global preferences, and the root `.cocommit.toml` policy without depending on terminal rendering or message formatting. The effective message policy is intentionally not consumed by the current domain model until Iteration 17; this preserves one active validation and formatting path.

Help and version exit before the interactive-stream and Git checks. Usage errors exit before terminal initialization. The executable maps usage errors to exit code `2`, while operational failures use `1` and successful cancellation uses `0`.

Terminal restoration is global and idempotent while raw mode is active. This lets normal cleanup, partial initialization errors, the panic hook, and supported Unix termination signals use the same recovery routine without hiding a panic's original report.
