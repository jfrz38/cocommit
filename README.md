# cocommit

`cocommit` is a small keyboard-driven terminal UI for creating Conventional Commit headers from already staged changes. It previews the message as you edit it, then delegates the commit to your installed Git CLI.

```text
feat(api)!: add authentication (#123)
```

## Requirements

- Git must be installed and available on `PATH`.
- Run the application from a non-bare Git working tree that has staged changes.
- A terminal that supports interactive input is required.
- Rust 1.94.1 is required when installing from source. The repository pins this version with `rust-toolchain.toml`.

## Install

Install from a local checkout:

```bash
cargo install --path . --locked
```

The installed `cocommit` executable must be on your `PATH`. To run directly from a checkout instead, use:

```bash
cargo run --locked
```

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

## Keyboard Controls

| Key | Behavior |
|---|---|
| Up / Down | Move between form fields and wrap around. In the type picker, change the highlighted item. |
| Tab / Shift+Tab | Move to the next or previous form field. |
| Enter | Open the type picker, advance from a text field, select the highlighted type, or submit when Commit is focused. |
| Ctrl+Enter | Submit from any form field. |
| Space | Toggle Breaking or Sign when focused; insert a space in text fields and the type-picker query. |
| F1 | Open or close keyboard help without losing form state. |
| Esc | Close the type picker or help; cancel from the form. |
| Ctrl+C | Cancel without invoking Git. |
| Backspace / Delete / Home / End | Edit a text field or type-picker query. |

The layout switches to a compact, vertically scrolling view in smaller terminals. Below `30x8`, it displays a resize instruction instead of the form.

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

Missing configuration uses `sign = true`. Set `sign = false` to omit the explicit `-S` flag. This does not force an unsigned commit: Git's own `commit.gpgSign` setting can still apply. Unknown keys and invalid TOML are reported as errors; cocommit never creates configuration files or directories.

## Git Behavior

Before opening the form, cocommit verifies that Git is available, the current directory is inside a usable working tree, and staged changes exist. Git remains authoritative after that check, so a changed index can still cause the final commit to fail.

After a valid submission, cocommit restores the terminal and runs `git commit` with inherited standard streams. This preserves normal Git output, hooks, credentials, signing prompts, and pinentry behavior. A hook or signing failure is therefore shown directly by Git and returns a failure from cocommit.

## v1 Limitations

- Only Conventional Commit headers are supported; bodies, footers, amend mode, and empty commits are not supported.
- cocommit does not stage or unstage files, show diffs or history, manage branches, push, or create pull requests.
- It does not replace Git identity, hooks, credentials, editors, or signing configuration.
- Repository-local configuration, configurable types or scopes, CLI prefills, dry runs, and AI-generated messages are out of scope.

## Development

Run the complete local quality suite with:

```bash
make check
```

See [docs/](docs/README.md) for the product, architecture, interaction, Git, and testing contracts.

## License

[MIT](LICENSE)
