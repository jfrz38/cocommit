# ADR 0013: Verify the release supply chain

## Status

Accepted.

## Decision

`cargo-deny` is the single automated control for RustSec advisories, licenses,
and dependency sources. Its explicit license allow-list is derived from the
locked graph; an exception must identify the crate or advisory, reason, owner,
and review date. Notices and unmaintained crates are reported as workspace
findings rather than silently ignored.

Dependabot proposes weekly, grouped updates for Cargo and GitHub Actions. Every
Action is committed by reviewed full SHA with a version comment. Actions are
preferred only where a short, inspectable shell command cannot safely replace
them.

The release candidate produces one `.crate`, a CycloneDX JSON SBOM, and a
SHA-256 manifest. GitHub Artifact Attestations provide build provenance for
those exact assets. The package is the verifiable artifact for this iteration;
the manifest naming permits target-specific binaries in Iteration 21 without a
new verification contract.

Validation and asset generation have read-only permissions. Transfer between
jobs uses temporary GitHub artifacts; tag/release creation alone receives
`contents: write`, and provenance alone receives `id-token: write` and
`attestations: write`. Publication uses OIDC through the protected
`crates-publish` Environment. No persistent crates.io token or second
publication flow is introduced.

## Consequences

- Releases fail rather than overwrite a tag or an asset with different bytes.
- A publish run verifies release assets, checksums, SBOM, provenance, tag, and
  package version before its irreversible approval boundary.
- Rulesets, Environment reviewers, and Trusted Publishing are GitHub settings
  with recorded external evidence; repository files document but cannot enforce
  them.
