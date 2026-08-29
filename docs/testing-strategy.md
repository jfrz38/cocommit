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

### App state and UI smoke tests

Test state transitions directly:

- Focus wraps with Tab and Shift+Tab.
- Space changes only focused boolean controls.
- Type picker opens and closes correctly.
- Filtering selects a standard type or custom query.
- Submit validation focuses the first invalid field.
- Cancel produces `AppAction::Cancel`.

Use `ratatui::backend::TestBackend` for narrow render smoke tests: normal terminal, small terminal, form mode, and picker mode. Assert that rendering does not panic; do not snapshot the entire screen.

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

## Quality commands

Every implementation phase should keep these passing:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Before release, run manual smoke tests on a real repository for cancel, valid unsigned commit, explicit signing, Git hook rejection, no staged changes, non-repository invocation, small terminal, and pasted text.
