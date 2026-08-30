# Testing strategy

## Goals

Tests concentrate on stable behavior with high regression risk: message construction, validation, configuration parsing, Git command construction, and state transitions. Full-screen visual snapshots are intentionally avoided because they are brittle and add little confidence for this small TUI.

## Unit tests

### Commit domain

`commit.rs` must have table-driven tests for:

- Minimal valid `type: message` header.
- Scope present, absent, and whitespace-only.
- Breaking marker with and without scope.
- Issue present and absent.
- All optional-field combinations.
- Custom types.
- Trimming of outer whitespace.
- Empty type and empty message.
- Invalid type delimiters and whitespace.
- Newline rejection in type, scope, and message.
- Issue parsing, non-numeric input, and `u64` overflow.

The expected rendered string must be asserted exactly.

### Configuration

Separate TOML parsing from filesystem access. Test:

- Defaults with no input.
- Empty TOML.
- `sign = true`.
- Malformed TOML.
- Unknown key rejection.
- Missing file produces defaults.
- Read failure carries the config path.

### Git module

Keep command construction observable without running a commit. Test:

- `-S` appears only for explicit signing.
- A disabled signing choice does not add `--no-gpg-sign`.
- A message with spaces and quotes is passed as one argument.
- Preflight exit code `0` means no staged changes.
- Preflight exit code `1` means staged changes exist.
- Other statuses are errors.
- Missing Git executable diagnostics identify Git and the command that could not start.
- Failed preflight diagnostics retain the command and Git stderr.

### CLI contract

Test the parser directly for no arguments, both help aliases, both version aliases, unknown arguments, and extra arguments. Black-box binary tests must verify that help and version work outside a repository with captured output, unknown arguments return exit code `2` without terminal sequences, and redirected standard streams return exit code `1` before Git or terminal initialization.

### App state and UI smoke tests

Test state transitions directly:

- Focus wraps with arrow keys, Tab, and Shift+Tab.
- Ctrl+Enter submits through normal validation outside the type picker.
- Space changes only focused boolean controls.
- Type picker opens and closes correctly.
- Filtering selects a standard type or custom query.
- Submit validation focuses the first invalid field.
- Cancel produces `AppAction::Cancel`.

Use `ratatui::backend::TestBackend` for narrow render smoke tests: normal terminal, compact terminal with vertical scroll, too-small terminal, form mode, and picker mode. Assert that rendering does not panic; do not snapshot the entire screen.

## Git integration test

An integration test uses `tempfile` and the real Git CLI:

1. Skip with a clear reason if `git --version` cannot run.
2. Create a temporary directory and run `git init`.
3. Set local `user.name` and `user.email`.
4. Create a file and stage it.
5. Invoke the production commit-command path without signing.
6. Run `git log -1 --format=%s`.
7. Assert the subject equals the expected Conventional Commit message.

No signing or hook behavior is tested automatically because those depend on host configuration. The product design intentionally delegates those workflows to Git.

The integration suite also covers preflight in an empty temporary repository, a staged repository, and a directory outside a repository.

## Quality commands

Every implementation phase should keep `make check` passing. It is the canonical local quality command and runs formatting, linting, tests, and a build with the lockfile enforced. Its component commands are:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --locked --all-targets --all-features
```

GitHub Actions runs `make check` on Ubuntu for pull requests to `develop` and `main`, every Monday at 06:00 UTC, and on manual dispatch. Windows and macOS run `cargo check --workspace --all-targets --all-features --locked` for pull requests to `main`, scheduled runs, and manual dispatch; this preserves portability coverage without depending on GNU Make.

Before release, run manual smoke tests on a real repository for cancel, valid unsigned commit, explicit signing, Git hook rejection, no staged changes, non-repository invocation, small terminal, and pasted text.

## Release workflow rehearsal

The release workflow must be tested from GitHub Actions before its first use. A manual dispatch runs the validation job only and must not create a tag or GitHub Release. Review its log for the checked-out SHA, clean worktree check, `make release-check-clean`, and resolved version.

For the automatic path, record evidence for these states in a private test repository or a non-release test version:

- No tag and no release creates both from the pushed `main` commit.
- Re-running the same workflow leaves the existing immutable tag and release unchanged.
- A later `main` merge with the same package version is a no-op.
- A tag without a release is completed without moving the tag.
- A tag outside `main` history and a release without a tag fail without modifying either resource.

The manual publish workflow must be observed to validate the latest release before its OIDC publication job starts. It must never be dispatched for the first `0.1.0` publication, which uses the documented temporary-token procedure.
