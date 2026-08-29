# ADR 0005: Create GitHub Releases from versioned main merges

## Status

Accepted.

## Context

The project needs a repeatable release boundary without making the normal version-bump workflow cumbersome. A merge to `main` is already reviewed and protected, and the package version is part of that reviewed change.

## Decision

A push to `main` validates the exact pushed commit and automatically creates `v<version>` plus its GitHub Release when neither exists. A manual dispatch performs the same unprivileged validation as a rehearsal and never creates a tag or release.

Tags are immutable. If a tag already exists, the workflow verifies that it is a lightweight tag on a commit reachable from the current `main` candidate. An existing matching release is a successful no-op; a tag without a release is completed. A release without a tag, an annotated tag, or a tag outside that history is an error and is never repaired by moving a tag.

crates.io publication remains a separate manual workflow. It validates and packages the selected latest GitHub Release before a distinct job authenticates and publishes it.

## Consequences

- A version bump merged to `main` is the explicit request for a GitHub Release.
- Later merges with the same version do not overwrite release metadata or move tags.
- Validation runs without write or OIDC permissions.
- Only the tag-and-release job has `contents: write`; only the crates.io publication job has `id-token: write`.
- Repository maintainers must configure any desired GitHub Environment approval rules outside this repository.
