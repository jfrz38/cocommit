# Architecture

`cocommit` is one Rust crate with explicit module boundaries rather than a
multi-crate framework.

```text
main -> cli, config, git, app, terminal, commit_workflow
config -> commit::policy
config -> settings
event -> app
terminal -> app, event, ui
ui -> app, commit
app -> commit, settings, staging
git -> staging
commit_workflow -> git, staging
```

## Modules

- `commit`: Conventional Commit draft, validation, rendering, and message policy.
- `app`: keyboard-driven composer state and interaction rules.
- `cli`: command-line parsing and usage text.
- `event`: Crossterm-to-application event mapping.
- `staging`: staged-change snapshot used by the application and UI.
- `settings`: UI settings shared by configuration and the application.
- `config`: TOML loading and merge logic that produces UI preferences and commit policy.
- `git`: explicit Git CLI commands, preflight checks, and index updates.
- `commit_workflow`: temporarily excludes selected staged files, commits the rest, and restores exclusions after a failed commit.
- `ui` and `terminal`: Ratatui rendering and Crossterm lifecycle respectively.

The commit domain does not depend on terminal, filesystem, or Git code. Git is
invoked with explicit arguments rather than through a shell, so hooks, signing,
credentials, and native Git output remain Git's responsibility.

Keep new behavior in the smallest module that owns it. Do not add traits,
dependency injection, or separate crates unless a second real implementation
needs them.
