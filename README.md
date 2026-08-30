# 🥥 cocommit

**Craft well-formatted Conventional Commit headers without leaving your terminal.**

[![CI](https://github.com/jfrz38/cocommit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jfrz38/cocommit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cocommit?logo=rust)](https://crates.io/crates/cocommit)
[![Downloads](https://img.shields.io/crates/d/cocommit)](https://crates.io/crates/cocommit)
[![License](https://img.shields.io/github/license/jfrz38/cocommit)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.94.1%2B-blue)](https://www.rust-lang.org)

`cocommit` is a small keyboard-driven terminal UI for reviewing and refining staged changes before creating Conventional Commit headers. It previews the message as you edit it, then delegates Git operations to your installed Git CLI.

The name combines **CO**nventional and **COMMIT**s, hence the coconut 🥥.

```text
feat(api)!: add authentication (#123)
```

## Why cocommit?

- Build valid Conventional Commit headers interactively.
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

The type picker also accepts custom types. Scope, breaking marker, issue number, and explicit signing are optional. The rendered header is:

```text
<type>(<scope>)<breaking>: <message> (#<issue>)
```

Optional parts are omitted when unset. The message is required, and all text fields are one line. The preview updates after every edit.

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
| Enter | Open the type picker, advance from a text field, select the highlighted type, or submit when Commit is focused. |
| Ctrl+Enter | Submit from any form field. |
| Space | Toggle Breaking or Sign when focused; include or exclude the selected staged file; insert a space in text fields and the type-picker query. The last included file cannot be excluded. |
| F1 | Open or close keyboard help without losing form state. |
| Esc | Close the type picker or help; cancel from the form. |
| Ctrl+C | Cancel without invoking Git. |
| Backspace / Delete / Home / End | Edit a text field or type-picker query. |

Typing while Type is focused opens the picker and filters standard types with a case-insensitive prefix search. For example, `d` and `do` select `docs` by default; a query with no matching prefix can be selected as a custom type. cocommit is keyboard-driven and does not support mouse input.

The expanded layout shows a bounded, scrollable staged-change list with file count, `A/M/D/R` states, insertions, deletions, binary-file count, and inclusion checkboxes. Focus `Staged changes` with `Tab`; `Up`/`Down` select a visible file and move to Sign or Commit at the list boundaries, while `Home`/`End` move to its first/last item. Press `Space` to include or exclude the selected file. The working tree and Git index remain unchanged until Commit, when all excluded files are unstaged once before creating the commit. The final included file is protected. The layout switches to a compact, vertically scrolling view in smaller terminals without leaving unused space between Commit and Preview. Below `30x8`, it displays a resize instruction instead of the form.

## Configuration

The optional global configuration file contains UI preferences only:

```toml
sign = true
```

It is read from `<config-dir>/cocommit/config.toml`:

| Platform | Typical path |
|---|---|
| Linux | `$XDG_CONFIG_HOME/cocommit/config.toml` or `~/.config/cocommit/config.toml` |
| macOS | `~/Library/Application Support/cocommit/config.toml` |
| Windows | `%APPDATA%\cocommit\config.toml` |

Missing configuration, or an unavailable platform configuration directory, uses `sign = true`. Set `sign = false` to omit the explicit `-S` flag. This does not force an unsigned commit: Git's own `commit.gpgSign` setting can still apply. Unknown keys, invalid TOML, and unreadable existing files are reported as errors before the interface opens. cocommit never creates configuration files or directories.

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

- Only Conventional Commit headers are supported; bodies, footers, amend mode, and empty commits are not supported.
- cocommit does not stage files, render full diffs or history, manage branches, push, or create pull requests.
- It does not replace Git identity, hooks, credentials, editors, or signing configuration.
- AI-generated messages, changelog generation, dry runs, copy-only mode, and CLI field prefills are not supported.
- Configuration is global only; repository-local settings and configurable types, scopes, or issue formatting are not supported.

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

Validate the package that will be published without uploading it:

```bash
make release-check
```

## License

[MIT](LICENSE)
