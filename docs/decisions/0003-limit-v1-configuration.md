# ADR 0003: Keep v1 configuration global and minimal

## Status

Superseded by ADR 0008 and ADR 0012.

## Context

Repository-local files, configurable type lists, scopes, and issue templates require precedence, merging, validation, and migration decisions. Only signing default is required for the initial workflow.

## Decision

Read an optional global TOML file from the platform config directory. Its v1 schema contains only `sign`.

## Consequences

- Setup is cross-platform and predictable.
- The product works without a config file.
- Future config features require an explicit compatibility and precedence design.
