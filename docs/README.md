# cocommit documentation

This directory is the implementation contract for `cocommit`. It records the product and technical decisions implemented by the Rust application.

Read the documents in this order:

1. [Product scope](product-scope.md)
2. [Architecture](architecture.md)
3. [Interaction design](interaction-design.md)
4. [Git and configuration](git-and-configuration.md)
5. [Testing strategy](testing-strategy.md)
6. [Implementation plan](implementation-plan.md)
7. [Terminal support](terminal-support.md)
8. [Roadmap](roadmap.md)
9. [Releasing](releasing.md)
10. [Supply-chain operations](supply-chain-operations.md)
11. [Compatibility and deprecation](compatibility.md)
12. [Architecture decisions](decisions/)

## Project status

- Initial planning: complete
- Rust implementation: complete through Phase 9
- Current work: pre-release roadmap; Iteration 22 documentation and governance
- Iteration 21: implementation complete; release-rehearsal evidence remains pending
- Release status: private pre-release; no public package or release exists
- Supported scope: complete Conventional Commit composition from already staged changes

## Guiding principles

- Keep the terminal interface compact, keyboard-driven, and understandable without documentation.
- Keep Conventional Commit formatting independent from the TUI.
- Invoke the installed Git CLI rather than reimplementing Git behavior.
- Prefer a small concrete design over framework-like abstractions.
- Do not expand into a general-purpose Git client.
