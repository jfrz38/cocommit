# ADR 0001: Use the installed Git CLI

## Status

Accepted.

## Context

Creating a commit could use `git2` or invoke Git directly. The product must preserve existing user signing, identity, hooks, credential, and Git configuration behavior.

## Decision

Use `std::process::Command` to invoke the installed `git` executable with explicit arguments. Execute the final commit only after restoring the terminal.

## Consequences

- Git behavior remains consistent with normal `git commit`.
- Hooks and interactive signing work in a normal terminal.
- Git must be installed and usable on the host.
- The application interprets only the small set of preflight statuses it requires.
