# ADR 0014: Distribute unsigned native binaries

## Status

Accepted.

## Decision

Each release candidate produces the Cargo package, one CycloneDX JSON SBOM, a
SHA-256 manifest, and native archives for these explicitly supported targets:

- `x86_64-unknown-linux-gnu` as `.tar.gz`.
- `x86_64-pc-windows-msvc` as `.zip`.
- `aarch64-apple-darwin` as `.tar.gz`.

Each archive has exactly one root directory named
`cocommit-<version>-<target>` containing `cocommit` or `cocommit.exe`,
`LICENSE`, and a target-specific `INSTALL.md`. Names, file order, ownership,
permissions, timestamps, and archive metadata are normalized so packaging is
deterministic for identical binary inputs. This does not claim that Rust
compilation itself is reproducible.

GitHub-hosted runners build the binary for each native target at the candidate
SHA, then a read-only job packages the archives. Native runners download and
execute the packaged bytes before the only `contents: write` job can create a
release. The checksum manifest and GitHub Artifact Attestations cover every
release asset.

Windows binaries are not Authenticode signed. macOS binaries are not signed
with Developer ID or notarized. This avoids developer accounts, certificates,
key custody, and recurring costs for `0.1.0`. Users may instead install from
crates.io with Cargo after the package is published.

## Consequences

- Windows SmartScreen and macOS Gatekeeper can warn about downloaded binaries.
- Linux targets other than x86_64 GNU, Windows ARM, and Intel macOS are not
  supported by release archives.
- Homebrew, Scoop, Winget, code signing, and notarization remain post-release
  work.
- The crates.io package is constrained to Cargo metadata, license, README
  files, and Rust sources, and its contents are checked before release.
