# Releasing cocommit

`cocommit` publishes one Rust crate. GitHub Releases contain generated notes
only; there are no prebuilt binaries, archives, checksums, or installers.

## Prerequisites

- The release candidate has been reviewed and merged into `develop`.
- `make check` and `make release-check-clean` pass from a clean checkout.
- The repository is public so crates.io visitors can access its source, issue
  tracker, and README links.
- Repository Actions may create pull requests, and the `crates-publish` GitHub
  Environment exists with the required reviewer protection.
- After the first publication, crates.io Trusted Publishing is configured for
  this repository's `publish.yml` workflow and the `crates-publish` Environment.

## Release flow

1. Run **Bump Version** from the Actions tab and choose `patch`, `minor`, or
   `major`. It creates a draft pull request against `develop` and regenerates
   `Cargo.lock`.
2. Finalize the matching dated entry in `CHANGELOG.md`, then review and merge
   the bump pull request into `develop`.
3. Merge a pull request from `develop` into `main`. Do not fast-forward `main`:
   the release workflow compares the candidate with its preceding `main` commit.
4. The **Release** workflow runs quality and package checks. On a push to
   `main`, `jfrz38/check-version-change` creates `v<version>` and a GitHub
   Release with generated notes only when `Cargo.toml` contains a changed,
   higher stable version. A manual **Release** dispatch validates only; it never
   creates a tag or release.
5. Run **Publish to crates.io** manually from `main`. It selects the latest
   stable GitHub Release, verifies its tag is an ancestor of `main`, uses
   `jfrz38/check-version-change` to ensure the version is unpublished and newer
   than crates.io's latest version, validates the tagged package, and publishes
   through the protected `crates-publish` Environment.

Never move, delete, or reuse a release tag. A release retry accepts an existing
tag only when it resolves to the current `main` commit.

## Initial publication

crates.io Trusted Publishing can only be configured after the crate exists. Once
`v0.1.0` has been created on `main`, publish it once from that immutable tag with
a temporary, publish-only crates.io token:

```sh
git fetch --tags origin
git switch --detach v0.1.0
make release-check-clean
cargo publish --locked --token "$CARGO_REGISTRY_TOKEN"
```

Revoke the token after publication. Then configure crates.io **Trusted
Publishing** for GitHub repository `jfrz38/cocommit`, workflow
`.github/workflows/publish.yml`, and environment `crates-publish`. The workflow
requests an OIDC token only in its protected publishing job.

## Recovery

Crates.io versions are immutable. If a published version must be withdrawn, yank
it rather than changing its tag:

```sh
cargo yank --version 0.1.0 --token "$CARGO_REGISTRY_TOKEN"
```

Publish a corrected, higher version through the normal bump and release flow.
