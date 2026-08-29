# ADR 0004: Use an explicit Submit focus instead of a modal

## Status

Accepted.

## Context

The user needs protection from accidental commits, but a confirmation dialog adds a repetitive extra step to a focused one-purpose tool.

## Decision

Make Commit a separately focusable action. Only Enter on that action submits; Enter in editable fields advances or selects.

## Consequences

- Submission remains intentional without an extra modal.
- Keyboard behavior stays compact and predictable.
- Validation failures retain the form and focus the invalid field.
