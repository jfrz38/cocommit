# Implementation plan

> Historical implementation record. The current pre-release work and release
> gates are maintained in [roadmap.md](roadmap.md).

## Phase 1: Scaffold

Create `Cargo.toml`, pin Rust 1.94.1 and edition 2024, add the selected dependencies, create the source modules, and version `Cargo.lock` because this is an application. Add `rust-toolchain.toml`, a Makefile for local quality checks, and the initial GitHub Actions CI workflow.

Completion criteria:

- `cargo check --workspace --all-targets --all-features --locked` succeeds.
- `make check` runs formatting, linting, tests, and a build with the locked dependency graph.
- Module boundaries match [architecture.md](architecture.md).
- CI runs the quality suite and release builds on Ubuntu, Windows, and macOS for release-boundary pull requests, scheduled runs, and manual runs.

## Phase 2: Commit domain

Implement `CommitDraft`, field-oriented validation errors, normalization, canonical rendering, and exhaustive unit tests.

Completion criteria:

- One formatter produces every preview and submitted header.
- All formatting and validation cases in [testing-strategy.md](testing-strategy.md) pass.

## Phase 3: Configuration

Implement `Config::default`, config path resolution, TOML parsing, and file-loading error context.

Completion criteria:

- Absent config behaves as `sign = true`.
- Invalid existing config fails before terminal initialization.

## Phase 4: Git preflight and commands

Implement repository and staged-change checks, commit argument construction, and status interpretation. Add command-construction unit tests.

Completion criteria:

- Non-repository and no-staged-change failures are clear.
- `-S` is added only when requested.
- No shell is used.

## Phase 5: Application state and events

Implement editable form state, focus navigation, boolean toggles, the searchable type picker, text sanitization, and submit validation behavior.

Completion criteria:

- State-transition tests cover all keyboard interactions described in [interaction-design.md](interaction-design.md).
- A custom type can be entered and selected.

## Phase 6: TUI and terminal lifecycle

Implement Ratatui layout and rendering, then `TerminalSession` for raw-mode and alternate-screen lifecycle. Add small-terminal and popup rendering smoke tests.

Completion criteria:

- Preview changes with all fields.
- Terminal restoration is attempted on normal exits and errors.
- Resize and key press events work without duplicate handling.

## Phase 7: Wire the executable

In `main`, perform preflight and config loading, run the UI, restore the terminal, then execute Git with inherited standard streams. Add the temporary-repository integration test.

Completion criteria:

- A successful commit exits cleanly with Git output visible.
- Git hook and signing failures are visible and return failure.
- Cancel does not invoke Git.

## Phase 8: Documentation and release readiness

Update the root `README.md` with installation, usage, configuration, keyboard controls, requirements, and v1 limitations. Run formatting, linting, tests, and manual smoke tests.

Completion criteria:

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass.
- Manual checks cover the cases in the testing strategy.

## Phase 9: Release pipeline

Automate GitHub Release creation from `main`, while keeping crates.io publication as an explicit manual workflow. Add a version-bump workflow that opens a reviewable pull request against the selected release branch.

Completion criteria:

- A manual version bump updates `Cargo.toml` and `Cargo.lock` in a draft pull request.
- A push to `main` validates the package and creates exactly one `v<version>` GitHub Release for a new stable package version.
- A manual publish checks out the latest release tag, validates its ancestry and version, then publishes that exact package to crates.io through Trusted Publishing.
- No long-lived crates.io token is stored in repository configuration.
- `make release-check` runs all quality checks and validates the Cargo package without publishing it.

## Dependency selection

| Crate | Purpose |
|---|---|
| `ratatui` | Layout, widgets, and terminal rendering. |
| `crossterm` | Cross-platform terminal backend and event input. |
| `serde` with `derive` | Typed config deserialization. |
| `toml` | TOML config parsing. |
| `dirs` | Cross-platform config directory discovery. |
| `tui-input` | Unicode-aware one-line editing and cursor behavior. |
| `anyhow` | Concise contextual errors at binary orchestration boundaries. |
| `tempfile` as dev-dependency | Isolated temporary Git repository integration tests. |

No async runtime, `git2`, argument parser, fuzzy matcher, snapshot framework, or logging framework is needed for v1.
