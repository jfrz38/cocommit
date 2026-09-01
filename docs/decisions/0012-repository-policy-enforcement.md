# ADR 0012: Enforce resolved repository message policy

## Status

Accepted.

## Decision

`CommitDraft` validates and renders with the resolved `MessagePolicy`; App preview, submission, terminal revalidation, and the Git boundary use that same policy.

Absent `[message].types` retains the built-in types as suggestions and accepts custom types. An explicitly configured `types` list is a case-sensitive allow-list and suppresses custom type choices. Scope suggestions are non-binding: they are selectable from the Scope field and custom or empty scopes remain valid.

Subject policy counts Unicode scalar values. `lowercase` and `uppercase` apply to the first Unicode alphabetic character; subjects without one pass that check. Terminal punctuation means one of `.`, `!`, or `?`. `forbid` rejects those endings and `require` requires one.

Issue input remains decimal digits only. The configured prefix and style render either ` (PREFIX123)` or ` PREFIX123`. Scope and issue remain optional because schema version 1 has no required-field policy.

Compatibility with commitlint is deliberately limited: `type-enum`, the documented subject length/case/punctuation subset map to message policy. Scope enums are suggestions, header-length, plugins, dynamic rules, and JavaScript/TypeScript config execution are unsupported. Git hooks remain the final authority.

## Consequences

- Preview and committed messages have one canonical formatter.
- Repository configuration cannot execute code.
- Policy feedback remains in the form before the terminal restores for Git.
