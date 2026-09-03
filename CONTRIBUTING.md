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
- Update user and technical documentation with the behavior.
- Record an ADR before a compatibility, security, configuration-format, or
  product-boundary decision.

Run the relevant checks before opening a pull request:

```bash
make check
make check-workflows
make supply-chain-check
```

For release, packaging, or documentation changes, also run `make release-check`.
Describe manual terminal checks, supported platform impact, and any deferred
release-gate evidence in the pull request.

## Pull requests

Keep pull requests reviewable, explain the user-visible effect, and link the
relevant roadmap iteration or ADR. Do not commit credentials, local paths,
release archives, crash dumps, or generated build output.

See [docs/README.md](docs/README.md) for the implementation contract and
[SECURITY.md](SECURITY.md) for private vulnerability reports.
