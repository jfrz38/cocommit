# Contributing to cocommit

## Local setup

Install Git and Rust 1.94.1. The repository pins its toolchain in
`rust-toolchain.toml`.

```bash
git clone https://github.com/jfrz38/cocommit.git
cd cocommit
make check
```

Use focused branches from `develop` and Conventional Commit messages. Keep a
change limited to its stated behavior; do not add general Git-client features.

## Change expectations

- Keep message formatting and validation independent from terminal rendering.
- Use the installed Git CLI through explicit argument arrays; do not add shell
  parsing or reimplement Git behavior.
- Keep repository message policy separate from global UI preferences.
- Add or update stable automated tests for behavior changes.
- Update the README, configuration guide, or changelog when the change affects them.

Run the relevant checks before opening a pull request:

```bash
make check
make check-workflows
make supply-chain-check
```

For release or packaging changes, also run `make release-check`.

## Pull requests

Keep pull requests reviewable and explain the user-visible effect. Do not commit
credentials, local paths, crash dumps, or generated build output.

See [architecture](docs/architecture.md) for module boundaries and
[SECURITY.md](SECURITY.md) for private vulnerability reports.
