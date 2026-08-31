# ADR 0009: Model complete Conventional Commit messages before editing them

## Status

Accepted.

## Context

The original domain model represented only a Conventional Commit header. The first release requires a canonical complete message, but multiline editing and footer controls belong to the following TUI iteration. The domain must therefore establish one transport and formatting contract before UI work begins.

## Decision

`CommitDraft` contains the existing header, an optional multiline body, and an ordered list of `Footer` values. `render_message` is the sole renderer for preview and Git submission. It joins present sections with exactly one blank line and emits every footer as `Token: value` in supplied order. Absent sections add no blank lines.

Body text is free-form and may contain paragraphs. Its outer whitespace is trimmed while internal line structure is retained. Footer values may span multiple non-blank lines; outer whitespace is trimmed and blank lines are rejected so the footer section remains unambiguous. Footer tokens must be ASCII letters or digits joined by hyphens. The only whitespace-containing token is uppercase `BREAKING CHANGE`.

`BREAKING CHANGE` and `BREAKING-CHANGE` are accepted as equivalent input and canonicalized to `BREAKING CHANGE`. A header `!` and one breaking footer may coexist because both are valid Conventional Commits signals. At most one breaking footer is permitted. Other trailers, including repeated `Co-authored-by`, remain ordered and repeatable; there is no closed token list.

The complete rendered message is passed as one `git commit -m` argument. No shell parsing, temporary message file, or alternate message transport is introduced. The current TUI continues to create header-only drafts until Iteration 16 exposes body and footer editing.

## Consequences

- Preview and submission can share one deterministic complete-message renderer.
- Future UI work edits structured data rather than parsing a free-text commit message.
- Repeated ordinary trailers remain possible while conflicting breaking semantics fail before Git runs.
- The public UI continues to support header composition only until its multiline controls are implemented.
