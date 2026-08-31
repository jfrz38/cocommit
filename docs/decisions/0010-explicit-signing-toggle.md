# ADR 0010: Make the signing toggle an explicit Git choice

## Status

Accepted.

Supersedes [ADR 0002](0002-signing-semantics.md).

## Context

The previous toggle added `-S` when enabled and otherwise deferred to Git. A user with `commit.gpgSign=true` therefore received a signing prompt even though the checkbox was unchecked. A binary control must not imply an unsigned commit while leaving signing enabled through an unseen Git setting.

## Decision

The UI is labelled `Sign commit`. When enabled, cocommit runs `git commit -S -m <message>`. When disabled, it runs `git commit --no-gpg-sign -m <message>`. The application no longer exposes a "use Git default" signing state.

The global `ui.sign` configuration and legacy global `sign` key retain their boolean shape: `true` initializes the toggle as enabled and `false` initializes it as disabled. Git hooks, identity, keys, credentials, pinentry, and signing implementation remain Git's responsibility.

## Consequences

- A disabled toggle reliably requests an unsigned commit, including when `commit.gpgSign=true` is configured globally or locally.
- A checked toggle reliably requests signing with `-S`.
- Users who need Git-default behavior must use Git directly until a future three-state control is justified.
