# 🥥 cocommit

**Craft complete Conventional Commit messages without leaving your terminal.**

[![CI](https://github.com/jfrz38/cocommit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jfrz38/cocommit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cocommit?logo=rust)](https://crates.io/crates/cocommit)
[![Downloads](https://img.shields.io/crates/d/cocommit)](https://crates.io/crates/cocommit)
[![License](https://img.shields.io/github/license/jfrz38/cocommit)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.94.1%2B-blue)](https://www.rust-lang.org)

`cocommit` is a small keyboard-driven terminal UI for reviewing and refining staged changes before creating complete Conventional Commit messages. It previews the message as you edit it, then delegates Git operations to your installed Git CLI.

The name combines **CO**nventional and **COMMIT**s, hence the coconut 🥥.

```text
feat(api)!: add authentication (#123)
```

## Why cocommit?

- Build valid Conventional Commit headers, bodies, and footers interactively.
- Preview the final message before committing.
- Review staged file states and aggregate line statistics, and unstage an accidental file without leaving the composer.
- Preserve Git hooks, signing, credentials, and native output.
- Run from anywhere inside your Git working tree.
- Stay entirely in the terminal.

## Install

Install `cocommit` from crates.io:

```bash
cargo install cocommit --locked
```

The installed `cocommit` executable must be on your `PATH`.

## Requirements

- Git must be installed and available on `PATH`.
- Run the application from a non-bare Git working tree that has staged changes.
- A terminal that supports interactive input is required.
- Rust 1.94.1 is required when installing from source. The repository pins this version with `rust-toolchain.toml`.
- The release-candidate terminal matrix and known keyboard limitations are documented in [Terminal support](docs/terminal-support.md).

## Usage

Stage the changes you want to commit, then run `cocommit` from any directory inside that working tree:

```bash
git add src/main.rs
cocommit
```

The form starts with the `feat` type and an empty message. It supports the standard Conventional Commit types:

```text
feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
```

The type picker also accepts custom types. Scope, breaking marker, issue number, and signing choice are optional. The rendered header is:

```text
<type>(<scope>)<breaking>: <message> (#<issue>)
```

Optional parts are omitted when unset. The subject is required and one line; Body accepts paragraphs. The Footers section opens a structured modal editor for ordered trailers and multiline values. The preview updates after every edit.

### Command-line options

```text
cocommit --help
cocommit --version
```

`-h` and `-V` are equivalent short options. Any other argument is rejected with exit code `2`; the interactive command accepts no arguments. Help and version work outside a repository and with redirected output.

The interactive command requires both standard input and standard output to be terminals. This prevents terminal control sequences from being emitted to a pipe or redirected file.

| Exit code | Meaning |
|---|---|
| `0` | Help or version displayed, form cancelled, or commit succeeded. |
| `1` | Configuration, preflight, terminal, or Git commit failure. |
| `2` | Invalid command-line usage. |

## Validation

All text fields are trimmed before validation and rendering. The type is required and cannot contain whitespace, `(`, `)`, `!`, or `:`. Scope cannot contain parentheses, and a non-empty issue must be a decimal integer. The message must be present and one line, but cocommit does not impose capitalization, punctuation, or tense rules. Interactive limits are 64 characters for type, 128 for scope, 512 for message, 20 for issue, and 4096 for a single paste.

Invalid submission keeps the form open, displays a field-specific error, and focuses the first invalid field. Pasted line breaks are converted to spaces. NUL, escape, and other control characters are rejected; rejected input leaves the field unchanged.

## Keyboard Controls

| Key | Behavior |
|---|---|
| Up / Down | Move between form fields and wrap around. In Staged changes, select the previous or next file; at either end, move to the adjacent form field. In the type picker, change the highlighted item. |
| Tab / Shift+Tab | Move to the next or previous form field. |
| Enter | Open the type picker, advance from a text field, insert a body/footer-value newline, edit a footer, select the highlighted type, or submit when Commit is focused. |
| Ctrl+Enter | Submit from any form field. |
| Space | Toggle Breaking or Sign when focused; include or exclude the selected staged file; insert a space in text fields and the type-picker query. The last included file cannot be excluded. |
| F1 | Open or close keyboard help without losing form state. |
| Esc | Close the type picker or help; cancel from the form. |
| Ctrl+C | Cancel without invoking Git. |
| Backspace / Delete / Home / End | Edit a text field or type-picker query. |

Typing while Type is focused opens the picker and filters standard types with a case-insensitive prefix search. For example, `d` and `do` select `docs` by default; a query with no matching prefix can be selected as a custom type. cocommit is keyboard-driven and does not support mouse input.

Focus `Footers` with `Tab`. `A` opens a picker with `BREAKING CHANGE`, `Closes`, `Fixes`, `Refs`, and `Co-authored-by`, plus any custom footer name. `Enter` edits the selected footer, `Space` starts a `BREAKING CHANGE` footer, `Delete` removes it, and `Ctrl+Up`/`Ctrl+Down` reorder it. In the footer modal, `Tab` changes Footer name, Value, and Save footer; `Enter` inserts a newline in Value and `Ctrl+Enter` saves. The preview can be focused and scrolled with `Up`/`Down`.

The expanded layout shows a bounded, scrollable staged-change list with file count, `A/M/D/R` states, insertions, deletions, binary-file count, and inclusion checkboxes. Focus `Staged changes` with `Tab`; `Up`/`Down` select a visible file and move to Sign or Commit at the list boundaries, while `Home`/`End` move to its first/last item. Press `Space` to include or exclude the selected file. The working tree and Git index remain unchanged until Commit, when all excluded files are unstaged once before creating the commit. The final included file is protected. The layout switches to a compact, vertically scrolling view in smaller terminals without leaving unused space between Commit and Preview. Below `30x8`, it displays a resize instruction instead of the form.

## Configuration

The optional global configuration file retains the legacy UI-preference form:

```toml
sign = true
```

It is read from `<config-dir>/cocommit/config.toml`:

| Platform | Typical path |
|---|---|
| Linux | `$XDG_CONFIG_HOME/cocommit/config.toml` or `~/.config/cocommit/config.toml` |
| macOS | `~/Library/Application Support/cocommit/config.toml` |
| Windows | `%APPDATA%\cocommit\config.toml` |

Missing configuration, or an unavailable platform configuration directory, uses `sign = true`. Set `sign = false` to initialize `Sign commit` as disabled; cocommit then passes `--no-gpg-sign`, overriding Git's `commit.gpgSign` setting for that commit.

Schema-versioned global preferences can hide optional sections while preserving the complete interface by default:

```toml
schema_version = 1

[ui]
sign = true

[ui.sections]
staged_changes = true
body = true
footers = true
issue = true
```

For repository conventions, add `.cocommit.toml` at the Git work-tree root. cocommit resolves that root through Git, so the same file applies when it runs from a subdirectory or linked worktree. Configuration precedence is built-in defaults, global configuration, then repository configuration; a future CLI layer will be higher priority. UI preferences remain global, while the repository file accepts only message policy:

```toml
schema_version = 1

[message]
types = ["feat", "fix", "docs"]
scope_suggestions = ["api", "tui"]

[message.subject]
max_length = 72
capitalization = "lowercase"
terminal_punctuation = "forbid"

[message.issue]
prefix = "PROJ-"
style = "plain"
```

Every `ui.sections` value defaults to `true`; they are global-only preferences, so repository `.cocommit.toml` files cannot hide an editor. Type, Scope, Breaking, Subject, Sign commit, Preview, and Commit remain visible. With `staged_changes = false`, preflight still requires staged files, but no exclusion or unstage operation is offered and Git commits every file in its index at submission.

An explicitly configured `types` list is an allowed-type list; without it, standard types remain suggestions and custom types are accepted. Scope suggestions remain non-blocking. Issue identifiers remain decimal numbers; `prefix` and `style` select a closed rendering convention. The Conventional Commits scope syntax is fixed as `type(scope): subject`; `[]` and `<>` are not supported. Policy applies to picker choices, validation, preview, and the rendered Git message.

New configuration files require `schema_version = 1`. The legacy global `sign` form remains supported but cannot be mixed with schema-versioned fields; migrate it manually to `[ui]\nsign = false` before adding policy. Unknown keys, unsupported versions, invalid TOML, and unreadable existing files are reported with their path before the interface opens. cocommit never creates or rewrites configuration files or directories.

## Git Behavior

Before opening the form, cocommit verifies that Git is available, the current directory is inside a usable working tree, and staged changes exist. It then reads a NUL-delimited index summary; unusual names are safely escaped for the terminal while their original paths are retained for Git. Excluded files are removed from the index with literal pathspecs immediately before the commit. This works from subdirectories and linked worktrees. Git remains authoritative after that check, so an external index change can still cause the final commit to fail.

After a valid submission, cocommit restores the terminal and runs `git commit` with inherited standard streams. This preserves normal Git output, hooks, credentials, signing prompts, and pinentry behavior. Commit messages are passed directly to Git without shell interpretation. A hook or signing failure is therefore shown directly by Git and returns a failure from cocommit. Cancelling restores the terminal and exits successfully without invoking Git.

## Troubleshooting

- `Git executable was not found`: install Git and make sure `git` is on `PATH` in the terminal where you run cocommit.
- A preflight error names the failing Git command and includes Git's stderr. Follow that output for `safe.directory`, permission, repository, or index problems.
- `standard input` or `standard output must be an interactive terminal`: run cocommit directly in a terminal instead of through a pipe, redirection, or non-interactive task runner. `--help` and `--version` remain usable in those contexts.
- cocommit restores its terminal modes after normal completion, panic, and on Unix `SIGHUP`, `SIGINT`, `SIGQUIT`, or `SIGTERM`. If the terminal is still corrupted, run `reset` on Unix or open a new terminal session.
- Job-control suspension is not supported while the form is open. Cancel cocommit before suspending it; resuming an externally suspended process is not guaranteed to restore the form state.

## v1 Limitations

- Amend mode and empty commits are not supported.
- cocommit does not stage files, render full diffs or history, manage branches, push, or create pull requests.
- It does not replace Git identity, hooks, credentials, editors, or signing configuration.
- AI-generated messages, changelog generation, dry runs, copy-only mode, and CLI field prefills are not supported.
- Repository message policies guide the form and are enforced before Git runs; Git hooks remain final authority.

## Development

Run directly from a local checkout:

```bash
cargo run --locked
```

Install the local version:

```bash
cargo install --path . --locked
```

Run the complete local quality suite with:

```bash
make check
```

Run the cross-platform equivalent and validate CI definitions after installing actionlint and ShellCheck:

```bash
make check-portability
make check-workflows
# Equivalent to the local CI checks, after installing actionlint and ShellCheck.
make ci
```

Validate the package that will be published without uploading it:

```bash
make release-check
```

## License

[MIT](LICENSE)
