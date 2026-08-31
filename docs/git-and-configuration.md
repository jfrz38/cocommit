# Git and configuration

## Git CLI strategy

`cocommit` invokes the user's installed `git` executable through `std::process::Command`. It does not use `git2`.

This preserves Git configuration, hooks, identities, credentials, signing behavior, and the familiar behavior of a normal `git commit`.

Commands must use argument arrays, never a shell:

```rust
Command::new("git")
    .args(["commit", "-S", "-m", &message])
    .status()
```

The complete commit message is one argument even when it includes spaces, quotes, Unicode, or line breaks.

## Preflight

Run preflight before initializing the TUI:

```bash
git rev-parse --is-inside-work-tree
git diff --cached --quiet
git diff --cached --name-status -z --find-renames
git diff --cached --numstat -z --find-renames
```

Interpretation:

| Check | Result | Behavior |
|---|---|---|
| `rev-parse` cannot start | Git is unavailable or cannot execute | Identify the command; identify a missing executable separately. |
| `rev-parse` fails or does not print `true` | Not a usable working tree | Identify the command and preserve Git stderr. |
| `diff --cached --quiet` exits `0` | No staged changes | Return a clear error. |
| `diff --cached --quiet` exits `1` | Staged changes exist | Open the TUI. |
| `diff --cached --quiet` exits otherwise | Git failure | Identify the command and preserve Git stderr. |

This naturally supports Git worktrees and subdirectories because Git resolves repository context. Bare repositories are rejected because `--is-inside-work-tree` is not `true`.

After staged changes are confirmed, cocommit reads a NUL-delimited summary with `--name-status` and `--numstat` output. This avoids shell parsing and preserves file-record boundaries for spaces, tabs, newlines, Unicode, and renames. Terminal rendering escapes control characters in names while retaining the original paths for later Git arguments.

When the user presses `Space` on a selected file, cocommit only changes that file's inclusion checkbox in memory. On submission, it restores the terminal and removes all excluded files with literal pathspecs before committing. Repositories with `HEAD` run `git --literal-pathspecs restore --staged -- <path...>`; an initial repository runs `git --literal-pathspecs update-index --force-remove -- <path...>`. Renames pass both old and new paths. The working tree is not changed. The final included file is protected so the commit precondition remains true.

The index can change after preflight. Git remains authoritative and may still reject the commit.

Diagnostics keep Git's stderr for failed preflight commands. This preserves useful context for permission errors, `safe.directory`, corrupt repositories, and other Git-owned checks without attempting to classify every Git error.

## Commit execution

After a valid submit:

1. Drop or explicitly restore `TerminalSession`.
2. Build the exact `git commit` argument list with the canonical header, body, and footer renderer.
3. Inherit stdin, stdout, and stderr from the parent process.
4. Execute Git synchronously.
5. Exit with success when its status is successful; otherwise return an error carrying Git's exit status.

Do not capture and re-render Git stderr in the TUI. A hook, GPG, SSH signing, or pinentry may need a normal terminal. Restoring first also ensures Git output is not lost in Ratatui's alternate screen.

The message model renders a present body and footer block with exactly one blank line between sections. Footer order is retained, and every footer is passed to Git inside the same `-m` argument as the header. `BREAKING CHANGE` and `BREAKING-CHANGE` are equivalent input tokens and render as `BREAKING CHANGE`; a header `!` may coexist with that footer.

## Signing semantics

The UI setting is an explicit request to sign or not sign the commit:

| `Sign commit` | Command |
|---|---|
| Enabled | `git commit -S -m <message>` |
| Disabled | `git commit --no-gpg-sign -m <message>` |

The binary toggle deliberately overrides `commit.gpgSign`: a disabled `Sign commit` requests an unsigned commit. cocommit has no "use Git default" state; use Git directly when that behavior is required.

## Configuration layers

The configuration file is resolved with `dirs::config_dir()`:

```text
<config-dir>/cocommit/config.toml
```

Typical paths are:

| Platform | Path |
|---|---|
| Linux | `$XDG_CONFIG_HOME/cocommit/config.toml` or `~/.config/cocommit/config.toml` |
| macOS | `~/Library/Application Support/cocommit/config.toml` |
| Windows | `%APPDATA%\cocommit\config.toml` |

The effective configuration merges built-in defaults, the optional global file, then the optional repository file. A future CLI override layer will have the highest precedence, but no CLI configuration arguments exist yet. Scalars and nested fields override only when present. Lists replace the complete lower-priority list; `scope_suggestions = []` intentionally clears inherited suggestions.

The global file owns individual UI preferences. Its legacy schema remains valid:

```toml
sign = true
```

```rust
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub ui: UiPreferences,
    pub message: MessagePolicy,
}
```

Rules:

- Missing global file or unavailable config directory: use defaults.
- The default enables `Sign commit`; legacy `sign = false` initializes an explicit unsigned request.
- Existing unreadable file: return an actionable error with its path.
- Invalid TOML, unsupported schema versions, and unknown keys: return a parse error rather than silently ignore a typo.
- Do not create config files or directories automatically.
- Git identity, keys, hooks, and signing infrastructure remain Git's responsibility.

New configuration files use versioned TOML. The global file may contain UI and message defaults:

```toml
schema_version = 1

[ui]
sign = true

[message]
types = ["feat", "fix", "docs", "chore"]
scope_suggestions = ["api", "tui"]

[message.subject]
max_length = 72
capitalization = "lowercase" # allow | lowercase | uppercase
terminal_punctuation = "forbid" # allow | forbid | require

[message.issue]
prefix = "PROJ-"
style = "plain" # parenthesized | plain
```

The repository policy is always `<work-tree-root>/.cocommit.toml`, where Git resolves the root for subdirectories and linked worktrees. It accepts only `[message]`, never `[ui]` or legacy `sign`:

```toml
schema_version = 1

[message]
types = ["feat", "fix", "docs"]
scope_suggestions = []

[message.subject]
max_length = 72
```

`types` is the future allowed-type list and cannot be empty. Scope suggestions are never a restriction. Issue identifiers will remain decimal numbers; `prefix` and `style` support common renderings without templates or regular expressions. The only supported scope syntax is `type(scope): subject`; `[]` and `<>` are deliberately not configurable.

Iteration 14 loads and validates these policies but does not apply them to the current picker, validation, preview, or rendered Git message. That enforcement is deferred to Iteration 17, preserving the current formatter as the only active path. The legacy global file must be migrated manually from `sign = false` to `[ui]\nsign = false` before adding schema-versioned fields. No file is rewritten automatically.

## Validation contract

Validation occurs before the terminal closes for a commit attempt:

- Type is required after trimming; it must be one line and cannot contain whitespace, `(`, `)`, `!`, or `:`.
- Scope is optional; empty trimmed scope becomes absent. A present scope cannot contain line breaks or parentheses.
- Message is required after trimming and must be one line.
- Issue is optional. A non-empty value must contain decimal digits and parse as `u64`.
- No arbitrary subject-length, capitalization, punctuation, or tense policy is imposed.

All text fields are normalized by trimming their outer whitespace when creating `CommitDraft` for rendering and validation.
