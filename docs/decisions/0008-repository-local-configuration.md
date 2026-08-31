# ADR 0008: Layer repository message policy over global preferences

## Status

Accepted.

## Context

Teams need versioned commit-message conventions while individual terminal preferences must remain local. ADR 0003 deliberately deferred this until the project could define precedence, compatibility, and validation rules.

## Decision

`cocommit` reads an optional `.cocommit.toml` from the Git work-tree root resolved by `git rev-parse --show-toplevel`. It does not derive the path from the current directory, so subdirectories and linked worktrees use their own root file consistently.

The effective configuration merges in this order:

1. Built-in defaults.
2. Global `<config-dir>/cocommit/config.toml`.
3. Repository `<work-tree-root>/.cocommit.toml`.
4. A documented future CLI-override layer.

Scalars and nested fields override only when present. Lists replace the preceding list in full; an empty scope-suggestion list deliberately clears inherited suggestions. An empty type list is invalid.

Global configuration owns `[ui]` preferences, including `sign`. Repository configuration owns only `[message]` policy. The legacy global `sign = true|false` format remains supported when `schema_version` is absent. New configuration uses `schema_version = 1`; unknown versions, unknown keys, mixed legacy/new global syntax, and message policy without a schema version fail with the file path and source error. Files are never rewritten or migrated automatically.

Message policy records configurable allowed types, scope suggestions, subject policies, and decimal issue identifier prefix/rendering style. Scope rendering remains the Conventional Commits `()` syntax only. Iteration 14 does not yet apply this policy to the picker, validation, preview, or Git message; Iteration 18 will do so through the single domain renderer and validator.

## Consequences

- Repositories can version their future conventions without sharing a user's signing preference.
- Empty or absent files preserve existing behavior, including the standard type list, `(#123)` issues, and signing enabled.
- Lists have predictable override semantics and can be intentionally cleared.
- Existing global `sign` users remain compatible, but must migrate manually to `[ui]` before adding schema-versioned policy.
- Non-parenthesized scope delimiters are intentionally unsupported because they are not Conventional Commits syntax.
