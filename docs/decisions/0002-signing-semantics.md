# ADR 0002: Signing toggle means explicit -S

## Status

Superseded by [ADR 0010](0010-explicit-signing-toggle.md).

## Context

A boolean UI cannot represent force signing, use Git defaults, and force no signing simultaneously. `commit.gpgSign` may cause Git to sign even when no `-S` flag is supplied.

## Decision

This decision was superseded before the first public release.

## Consequences

- See ADR 0010 for the current explicit signed-or-unsigned behavior.
