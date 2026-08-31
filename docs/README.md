# cocommit documentation

This directory is the implementation contract for `cocommit`. It records the product and technical decisions implemented by the Rust application.

Read the documents in this order:

1. [Product scope](product-scope.md)
2. [Architecture](architecture.md)
3. [Interaction design](interaction-design.md)
4. [Git and configuration](git-and-configuration.md)
5. [Testing strategy](testing-strategy.md)
6. [Implementation plan](implementation-plan.md)
7. [Roadmap](roadmap.md)
8. [Releasing](releasing.md)
9. [Architecture decisions](decisions/)

## Project status

- Initial planning: complete
- Rust implementation: complete through Phase 9
- Current work: pre-release roadmap, Iterations 10 through 15 complete; Iteration 16 multiline and footer TUI is next
- Release status: private pre-release; no public package or release exists
- Supported scope: Conventional Commit header creation from already staged changes; complete-message domain ready for the next TUI iteration

## Guiding principles

- Keep the terminal interface compact, keyboard-driven, and understandable without documentation.
- Keep Conventional Commit formatting independent from the TUI.
- Invoke the installed Git CLI rather than reimplementing Git behavior.
- Prefer a small concrete design over framework-like abstractions.
- Do not expand into a general-purpose Git client.
