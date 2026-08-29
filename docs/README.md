# cocommit documentation

This directory is the implementation contract for `cocommit`. The project is planned, but not implemented yet.

Read the documents in this order:

1. [Product scope](product-scope.md)
2. [Architecture](architecture.md)
3. [Interaction design](interaction-design.md)
4. [Git and configuration](git-and-configuration.md)
5. [Testing strategy](testing-strategy.md)
6. [Implementation plan](implementation-plan.md)
7. [Architecture decisions](decisions/)

## Project status

- Planning: complete
- Rust implementation: not started
- Supported scope: Conventional Commit header creation from already staged changes

## Guiding principles

- Keep the terminal interface compact, keyboard-driven, and understandable without documentation.
- Keep Conventional Commit formatting independent from the TUI.
- Invoke the installed Git CLI rather than reimplementing Git behavior.
- Prefer a small concrete design over framework-like abstractions.
- Do not expand into a general-purpose Git client.
