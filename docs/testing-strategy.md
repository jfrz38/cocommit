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
- Header-only, body-only, footer-only, and combined complete messages.
- Exact blank-line separation, body paragraphs, footer ordering, and multiline footer values.
- `BREAKING CHANGE` and `BREAKING-CHANGE` normalization, coexistence with `!`, and duplicate breaking-footer rejection.
- Repeatable trailers such as `Co-authored-by`, invalid footer tokens, empty footer values, and blank footer-value lines.

The expected rendered string must be asserted exactly.

### Configuration

Separate TOML parsing from filesystem access. Test:

- Defaults with no input and legacy global `sign` compatibility.
- `ui.sections` defaults, independent partial overrides, and all-four-hidden configuration.
- Schema-1 global and repository parsing, including all nested tables.
- Field-by-field precedence, list replacement, and intentionally empty scope suggestions.
- Rejection of missing, mixed, unsupported, and future schema versions.
- Unknown key rejection at every table level, including unknown `ui.sections` keys and every `[ui]` table in repository configuration.
- Missing file produces defaults; invalid or unreadable files carry their path and source error.
- Empty type lists and zero subject limits are rejected.
- Git root resolution from a repository root, subdirectory, and linked worktree.

### Git module

Keep command construction observable without running a commit. Test:

- An enabled signing choice adds `-S`.
- A disabled signing choice adds `--no-gpg-sign`, including when `commit.gpgSign` is enabled.
- A message with spaces and quotes is passed as one argument.
- A complete message with Unicode and line breaks is passed unchanged as one argument.
- Preflight exit code `0` means no staged changes.
- Preflight exit code `1` means staged changes exist.
- Other statuses are errors.
- Missing Git executable diagnostics identify Git and the command that could not start.
- Failed preflight diagnostics retain the command and Git stderr.
- NUL-delimited `--name-status` parsing for additions, modifications, deletions, renames, and unusual path characters.
- NUL-delimited `--numstat` parsing for text, binary, and renamed files.
- Literal unstage argument construction for normal and initial repositories, including both rename paths.

### CLI contract

Test the parser directly for no arguments, both help aliases, both version aliases, unknown arguments, and extra arguments. Black-box binary tests must verify that help and version work outside a repository with captured output, unknown arguments return exit code `2` without terminal sequences, and redirected standard streams return exit code `1` before Git or terminal initialization.

On Unix, `tests/terminal_pty.rs` runs the compiled binary in a pseudo-terminal. It covers normal cancellation and verifies that `SIGTERM` restores alternate screen, bracketed paste, and cursor sequences before exiting. Redirected-stream coverage remains the deterministic initialization-failure boundary on every platform.

### App state and UI smoke tests

Test state transitions directly:

- Focus wraps with arrow keys, Tab, and Shift+Tab.
- Ctrl+Enter submits through normal validation outside the type picker.
- Space changes focused boolean controls, changes the selected file's inclusion state, and protects the final included file.
- Type picker opens and closes correctly.
- Filtering selects a standard type or custom query.
- Submit validation focuses the first invalid field.
- Body paste preserves normalized line breaks, while one-line fields still collapse them.
- Footer creation, editing, cancellation, deletion, breaking-change shortcut, and reordering preserve the canonical preview.
- Preview opens an expanded read-only view, preserves it through help, bounds scrolling to wrapped Unicode content, and resets its position when the draft changes.
- Focus order is the candidate order filtered by each combination of Body, Footers, Issue, and Staged changes visibility; Tab, Shift+Tab, arrows, Preview boundaries, and wrapping remain complete.
- Hidden sections cannot receive focus, a validation error, an editor, picker, modal, help entry, key hint, or a section-specific keyboard action.
- Hidden Body, Footers, and Issue contribute absent values; all-visible defaults retain the existing form order and behavior.
- Hidden Staged changes creates no exclusion state, keeps every staged file included, and never routes an action to unstage.
- Cancel produces `AppAction::Cancel`.
- NUL, escape, and other control characters are rejected from edits and paste without changing the field.
- Field and paste limits reject the whole input without partial insertion, including Unicode input counted as characters rather than bytes.

Use `ratatui::backend::TestBackend` for narrow render smoke tests: normal terminal, compact terminal with vertical scroll, too-small terminal, form mode, picker mode, footer modal, expanded preview, long bodies, many footers, wide Unicode text, and a long staged-change list. Cover every individually hidden section and the all-hidden configuration in full and compact layouts; assert hidden rows, help, hints, and modal entry points are absent. Assert that rendering does not panic; do not snapshot the entire screen.

## Git integration test

An integration test uses `tempfile` and the real Git CLI:

1. Skip with a clear reason if `git --version` cannot run.
2. Create a temporary directory and run `git init`.
3. Set local `user.name` and `user.email`.
4. Create a file and stage it.
5. Invoke the production commit-command path without signing.
6. Run `git log -1 --format=%s`.
7. Assert `git log --format=%B` equals the expected complete Conventional Commit message, including its body and footers.

Add a multiple-staged-file scenario with `staged_changes = false`; assert the commit contains all staged files and no unstage command is constructed.

No signing or hook behavior is tested automatically because those depend on host configuration. The product design intentionally delegates those workflows to Git.

The integration suite also covers preflight in an empty temporary repository, a staged repository, and a directory outside a repository. A staged-context scenario creates additions, modifications, deletions, renames, and Unicode names, then verifies the summary against Git. Deferred-unstage scenarios cover all common change kinds, literal path characters, working-tree preservation, and an initial repository without `HEAD`.

## Quality commands

Every implementation phase should keep `make check` passing. It is the canonical local quality command and runs formatting, linting, tests, and a build with the lockfile enforced. Its component commands are:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo build --locked --all-targets --all-features
```

Run `make check-portability` before changes to cross-platform behavior; it uses the release profile for the final build. After installing actionlint and ShellCheck, run `make check-workflows` to validate workflow YAML, embedded shell fragments, full-SHA Action references, comments, privilege boundaries, Environment, and artifact/provenance contract. Install the fixed cargo-deny version with `make cargo-deny-install`, then run `make supply-chain-check` for advisories, licenses, and sources. `make ci` combines all local CI-equivalent checks.

The `Makefile` is the source of truth for reusable quality commands: `make check-portability` runs formatting, Clippy, tests, and a release build, while `make check-workflows` runs actionlint with ShellCheck. GitHub Actions uses `check` and `check-portability` on Ubuntu. Windows and macOS mirror `check-portability` with direct Cargo commands only because GNU Make is not guaranteed on Windows runners. CI bootstraps actionlint and ShellCheck itself, then delegates their execution to `make check-workflows`. Every runner verifies `git --version` first, so the real-Git integration suite cannot silently skip in CI. All three operating systems run for pull requests to `develop` and `main`, scheduled runs, and manual dispatch.

Before release, run manual smoke tests on a real repository for cancel, valid unsigned commit, explicit signing, Git hook rejection, no staged changes, non-repository invocation, small terminal, bounded pasted text, and forced termination. Run the default configuration, each individually hidden optional section, all sections hidden, and a compact terminal case; verify no hidden control can be reached through navigation or help. The complete-message check must enter a multiline body and an ordered multiline footer through the footer modal, then compare `git log -1 --format='%B'` with Preview. With Staged changes hidden, stage multiple files and verify all are committed. On Unix, verify restoration after a supported external termination signal; on all platforms, verify cancellation with `Ctrl+C` through the event loop. Record the cross-platform evidence and keyboard limitations in [Terminal support](terminal-support.md).

From Git Bash or zsh at the project root, create an isolated manual-test repository:

```bash
project_root="$PWD"
sandbox="$(mktemp -d)"
git -C "$sandbox" init
git -C "$sandbox" config user.name "Cocommit Test"
git -C "$sandbox" config user.email "cocommit@example.com"
printf 'visual test\n' > "$sandbox/demo.txt"
git -C "$sandbox" add demo.txt
(cd "$sandbox" && cargo run --quiet --locked --manifest-path "$project_root/Cargo.toml")
git -C "$sandbox" log -1 --format='%B'
git -C "$sandbox" show --stat --oneline HEAD
```

## Release workflow rehearsal

The release workflow must be tested from GitHub Actions before its first use. A manual dispatch runs the validation job only and must not create a tag or GitHub Release. Review its log for the checked-out SHA, clean worktree check, `make release-check-clean`, resolved version, package, SBOM, checksums, and artifact transfer.

For the automatic path, record evidence for these states in a private test repository or a non-release test version:

- No tag and no release creates both from the pushed `main` commit.
- Re-running the same workflow leaves the existing immutable tag and release unchanged.
- A later `main` merge with the same package version is a no-op.
- A tag without a release is completed without moving the tag.
- A tag outside `main` history and a release without a tag fail without modifying either resource.
- Identical assets are accepted on rerun; an existing asset with different bytes fails.
- Tag ruleset update/deletion denial, Environment approval and cancellation before approval are recorded from a private rehearsal.
- `sha256sum -c`, CycloneDX inspection, and `gh attestation verify` succeed for the candidate SHA.

The manual publish workflow must be observed to validate the latest release before its OIDC publication job starts. It must never be dispatched for the first `0.1.0` publication, which uses the documented temporary-token procedure.
