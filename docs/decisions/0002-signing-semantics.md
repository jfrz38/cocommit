# ADR 0002: Signing toggle means explicit -S

## Status

Accepted.

## Context

A boolean UI cannot represent force signing, use Git defaults, and force no signing simultaneously. `commit.gpgSign` may cause Git to sign even when no `-S` flag is supplied.

## Decision

The `Sign (-S)` toggle adds `-S` only when enabled. When disabled, cocommit omits both `-S` and `--no-gpg-sign`.

## Consequences

- Disabled does not promise an unsigned commit.
- Existing Git signing configuration remains effective.
- A future three-state setting can add force-unsigned behavior only when justified.
