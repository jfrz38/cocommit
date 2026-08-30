# ADR 0006: Keep a minimal CLI contract and explicit exit codes

## Status

Accepted.

## Context

`cocommit` is primarily interactive, but users and automation need predictable help, version, failure, and cancellation behavior without initializing a terminal or requiring a Git repository.

## Decision

The binary accepts no arguments, `-h`/`--help`, and `-V`/`--version`. Any other argument or combination is a usage error. Help and version are handled before checking the working directory, Git, configuration, or terminal streams.

Exit code `0` means successful help/version output, cancellation, or a successful Git commit. Exit code `2` means a usage error. Exit code `1` means every operational failure, including non-interactive standard streams, configuration, Git preflight, terminal lifecycle, and Git commit failures.

The implementation uses a small local parser rather than an argument-parser dependency. The normal interactive command requires both standard input and standard output to be terminals before raw mode can be initialized.

## Consequences

- Scripts can reliably obtain help and version through redirected output.
- Invalid arguments cannot accidentally open the TUI.
- Diagnostics identify the Git preflight command and preserve its stderr when Git starts but rejects the repository.
- Future CLI options require an explicit compatibility decision and tests for their exit behavior.
