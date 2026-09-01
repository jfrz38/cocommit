# Supply-chain operations

## Policy and evidence

Run `make supply-chain-check` after dependency changes. It enforces the
locked `deny.toml` policy for RustSec advisories, licenses, and sources. Do not
add broad ignores: every exception must name the affected crate or advisory,
the risk owner, reason, and review date.

For every private release rehearsal, record the workflow run URLs, candidate
SHA, package version, tool versions, checksum verification output, attestation
verification output, and screenshots of the tag ruleset and Environment gate.

## Advisory or compromise response

1. Triage the advisory or suspected compromise and identify affected versions,
   reachable use, release assets, and users at risk.
2. Patch or update the dependency, then run the complete quality and
   supply-chain checks. A temporary exception is allowed only when documented
   in `deny.toml` with owner and review date.
3. If credentials may be exposed, revoke the affected token or identity and
   disable the relevant workflow or Environment until it is safe. GitHub
   attestations are immutable evidence; communicate that an attested asset is
   withdrawn rather than attempting to alter its provenance.
4. Yank a published crates.io version when appropriate. Mark or withdraw the
   GitHub Release, but retain its tag and evidence for auditability.
5. Publish a corrected, higher version. Never reuse a tag, version, or replace
   an existing asset. Regenerate the `.crate`, SBOM, SHA256SUMS, and provenance
   from the new candidate.
6. Communicate scope, remediation, and upgrade guidance. Preserve incident
   timeline, run URLs, checksums, attestations, and approval evidence.

## GitHub settings checklist

Configure these controls outside the repository and test them in a private
repository first:

- Create a ruleset for `refs/tags/v*` that blocks update and deletion for
  normal maintainers. Permit creation only to the minimal Create Release actor
  or bypass list. Prove a normal maintainer cannot move or delete a tag while
  the workflow can create a new one.
- Create Environment `crates-publish` with required reviewers, self-review
  prevention when available, and deployment branches restricted to `main`.
  Store no crates.io token in repository or Environment secrets.
- After the initial manual `0.1.0` publication, configure crates.io Trusted
  Publishing for `jfrz38/cocommit`, `publish.yml`, and Environment
  `crates-publish`. The initial token is narrowly scoped, ephemeral, and
  revoked immediately after use.
