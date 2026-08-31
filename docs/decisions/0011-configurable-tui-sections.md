# ADR 0011: Make optional TUI sections explicit global preferences

## Status

Accepted.

## Context

The complete-message composer offers optional body, footer, issue, and staged-context workflows. Showing every workflow to every user makes the form taller and distracts from a simple header-only commit, while repository conventions must not be weakened by one user's display preference.

Hiding staged context has a consequential Git meaning: the user can no longer exclude files from the prepared index. The design must make that behavior explicit without removing the mandatory preflight that prevents empty commits.

## Decision

Global configuration owns four independent visibility preferences:

```toml
schema_version = 1

[ui.sections]
staged_changes = true
body = true
footers = true
issue = true
```

All omitted values default to `true`. Type, Scope, Breaking, Subject, Sign commit, Preview, and Commit are always visible. The effective focus order is the established candidate order filtered by visible optional sections. A hidden section has no rendered space, focus target, help text, keyboard action, picker, or modal.

Body, Footers, and Issue start empty when hidden and cannot receive application validation focus. When Staged changes is hidden, Git preflight still requires staged content, but the application creates no exclusion state and never invokes Git unstage; the final commit uses all files present in Git's index when the terminal closes.

Repository-local `.cocommit.toml` remains limited to `[message]` policy and rejects `[ui]`. Future repository policies that require an optional message part must make its editor available or report an actionable configuration conflict; they must not silently override or ignore the user's UI preference.

These additive, optional global keys remain part of `schema_version = 1` before the first public release. The legacy global `sign` key remains valid only in its existing legacy form and cannot be mixed with schema-versioned fields.

## Consequences

- A default or existing installation retains the complete current UI.
- The application must centralize visible-section and focus derivation rather than maintaining fixed navigation branches.
- Tests must cover all section combinations, including layouts small enough to expose focus and scrolling errors.
- A hidden staged-context section deliberately trades file-level exclusion for a smaller interface; it does not stage, unstage, or alter working-tree files.
- The later working-tree-context preference may choose `staged` or `all`; it does not add another `hidden` state.
