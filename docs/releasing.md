# Releasing cocommit

> **Pre-release policy:** `cocommit` must not create a tag, GitHub Release, or crates.io package until Iterations 10 through 22 and the `0.1.0` release gate in the [roadmap](roadmap.md) are complete. The workflows below describe the Phase 9 implementation and will be hardened before first use. In particular, do not promote the current `0.1.0` version to `main` while `release.yml` still creates a release on every push to `main`.

`cocommit` uses three separate GitHub Actions workflows so each release boundary is explicit:

1. **Bump Version** creates a draft version-bump pull request.
2. **Create Release** runs after a push to `main` and creates the matching GitHub Release.
3. **Publish Package** is run manually from `main` and publishes the latest GitHub Release to crates.io.

The package version in `Cargo.toml`, its `v<version>` tag, its GitHub Release, and the crates.io version must all match.

## Prepare a version

In GitHub Actions, run **Bump Version** and select `patch`, `minor`, or `major`. Its default base branch is `develop`. The workflow opens a draft pull request named `chore/bump-version-<version>` and updates both `Cargo.toml` and `Cargo.lock`.

Review and merge that pull request into `develop`, then merge `develop` into `main`. Do not edit the version directly on `main`.

The repository must allow GitHub Actions to create pull requests at **Settings > Actions > General > Workflow permissions**. Enable **Allow GitHub Actions to create and approve pull requests**.

## Create the GitHub Release

Every push to `main` runs **Create Release**. It runs `make release-check`, reads the stable SemVer version from `Cargo.toml`, creates `v<version>` at that exact commit, and generates GitHub release notes.

If the matching GitHub Release already exists, the workflow does nothing. If a tag with that version points to a different commit, it fails instead of moving the tag.

The workflow does not publish to crates.io.

## Publish to crates.io

After reviewing the GitHub Release, open **Publish Package** in GitHub Actions and run it from `main`. It:

1. Resolves the latest GitHub Release.
2. Checks out its immutable tag.
3. Verifies that the tag is reachable from `main` and that the tag version matches `Cargo.toml`.
4. Verifies the version is not already present on crates.io.
5. Runs `make release-check`.
6. Publishes with a temporary crates.io token obtained through GitHub OIDC.

The workflow never publishes the mutable `main` checkout and does not use a long-lived registry token.

## Trusted Publishing setup

crates.io Trusted Publishing can only be configured after the crate's initial publication. Once `cocommit` exists on crates.io, an owner must configure it at **crates.io > cocommit > Settings > Trusted Publishing** with:

| Setting | Value |
|---|---|
| CI/CD platform | GitHub Actions |
| Repository owner | `jfrz38` |
| Repository name | `cocommit` |
| Workflow filename | `publish.yml` |
| Environment | Leave empty |

The `publish.yml` workflow requests `id-token: write`; `rust-lang/crates-io-auth-action` exchanges that identity for a short-lived token and revokes it when the job finishes.

## Initial 0.1.0 publication

The first publication cannot use Trusted Publishing. After `v0.1.0` is created:

1. Run `make release-check` on the release commit.
2. Create a crates.io API token scoped only to publish new crates.
3. Run `cargo publish --locked --token <token>` from that release commit.
4. Revoke the token immediately after a successful publication.
5. Configure Trusted Publishing as described above.

The GitHub Release is still created automatically; only this initial registry publication is manual. Future versions use **Publish Package** and require no stored crates.io secret.

## Local release validation

Before merging a release bump, run:

```bash
make release-check
```

This runs formatting, Clippy, tests, a build, and `cargo publish --dry-run --locked --allow-dirty`. It packages and validates the crate but never uploads it. `--allow-dirty` lets maintainers validate a release bump before its pull request is committed; the publish workflow still checks out and publishes a clean release tag.
