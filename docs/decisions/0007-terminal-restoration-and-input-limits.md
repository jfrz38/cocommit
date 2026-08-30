# ADR 0007: Restore terminal state globally and bound interactive input

## Status

Accepted.

## Context

The normal terminal-session cleanup path cannot run when the process panics or receives a termination signal. Unbounded pasted text and terminal control characters also risk confusing rendering, validation, and terminal state.

## Decision

Terminal state is tracked globally only while raw mode is active. A single idempotent restoration routine attempts to show the cursor, disable bracketed paste, leave the alternate screen, and disable raw mode. Normal completion, partial initialization failures, `Drop`, and the panic hook use that same routine. The panic hook restores first and then calls the previous hook so the original panic report remains visible.

On Unix, a dedicated signal thread restores state and terminates with the conventional `128 + signal` status for `SIGHUP`, `SIGINT`, `SIGQUIT`, and `SIGTERM`. `Ctrl+C` typed in the raw-mode terminal remains a normal application cancellation because raw mode delivers it as an input event. Windows has no equivalent portable signal set in this release; its interactive `Ctrl+C` path remains supported.

Job-control suspension is deliberately unsupported during the form. Raw mode prevents terminal-generated `Ctrl+Z` suspension, and users should cancel before suspending the process. Resuming an externally suspended process is not guaranteed to reinitialize the form terminal state.

One-line fields use character-count limits: type 64, scope 128, subject 512, and issue 20. A paste is capped at 4096 characters. Newlines in a paste become one space per contiguous line break; NUL, escape, and all other control characters are rejected. A rejected edit or paste leaves the field unchanged and displays concise feedback.

## Consequences

- Cleanup has one implementation across ordinary and abnormal paths.
- The process preserves panic diagnostics and does not leave a supported Unix terminal in raw mode after a handled termination signal.
- Input size and character handling are deterministic and testable without coupling validation to rendering.
- Multiline bodies and larger field limits will require an explicit future decision.
