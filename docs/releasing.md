# Releasing cocommit

> **Pre-release policy:** `cocommit` must not create a tag, GitHub Release, or crates.io package until Iterations 10 through 23 and the `0.1.0` release gate in the [roadmap](roadmap.md) are complete. The workflows below are hardened for the approved automatic GitHub Release flow, but must be rehearsed privately before their first use.

`cocommit` uses three separate GitHub Actions workflows so each release boundary is explicit:

1. **Bump Version** creates a draft version-bump pull request.
2. **Create Release** validates a push to `main` and creates the matching GitHub Release when the version is new.
3. **Publish Package** is run manually from `main` and publishes the latest GitHub Release to crates.io.

The package version in `Cargo.toml`, its `v<version>` tag, its GitHub Release, and the crates.io version must all match.

## Prepare a version

In GitHub Actions, run **Bump Version** and select `patch`, `minor`, or `major`. Its default base branch is `develop`. The workflow opens a draft pull request named `chore/bump-version-<version>` and updates both `Cargo.toml` and `Cargo.lock`.

Review and merge that pull request into `develop`, then merge `develop` into `main`. Do not edit the version directly on `main`.

The repository must allow GitHub Actions to create pull requests at **Settings > Actions > General > Workflow permissions**. Enable **Allow GitHub Actions to create and approve pull requests**.

## Create the GitHub Release

Every push to `main` runs **Create Release**. An unprivileged job checks out the exact pushed commit, requires a clean worktree, runs `make release-check-clean`, and uses `check-version-change` to compare the `Cargo.toml` version with the preceding commit. It uses the action's validated local stable SemVer as the release version. Only after that succeeds can the separate `contents: write` job create `v<version>` at that commit and generate GitHub release notes.

The tag is immutable. If the matching GitHub Release already exists, the workflow verifies its lightweight tag remains on a commit reachable from the pushed `main` commit and does nothing. A later merge with the same package version therefore does not rewrite a release. If a tag exists without a release, the workflow creates only the missing release. A release without a tag, an annotated tag, or a tag outside `main` history fails instead of moving or replacing anything.

The workflow does not publish to crates.io.

## Rehearse the release validation

Run **Create Release** manually from the commit or branch to validate. Manual dispatch executes only the unprivileged validation job; it cannot create a tag or GitHub Release. Confirm in the Actions UI that `create-release` is skipped and that the validation log reports the expected SHA and version.

Run the normal `main` path only after recording the rehearsal evidence. Before its first use, use a private test version or repository to verify first creation, repeated execution, later merges with the same version, tag-only recovery, and conflicting-state failures.

## Publish to crates.io

After reviewing the GitHub Release, open **Publish Package** in GitHub Actions and run it from `main`. Its unprivileged validation job selects the latest published stable release, excluding drafts and prereleases, then:

1. Resolves the latest GitHub Release.
2. Checks out its immutable tag.
3. Verifies that the tag is reachable from `main` and that the tag version matches `Cargo.toml`.
4. Verifies the version is not already present on crates.io.
5. Runs `make release-check-clean` and records the package checksum.

The separate publication job checks out that exact tag, repackages without package verification, requires the checksum to match, and then publishes with a temporary crates.io token obtained through GitHub OIDC.

The workflow never publishes the mutable `main` checkout and does not use a long-lived registry token. Only its publication job has `id-token: write`.

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

This runs formatting, Clippy, tests, a build, and `cargo publish --dry-run --locked --allow-dirty`. It packages and validates the crate but never uploads it. `--allow-dirty` lets maintainers validate a release bump before its pull request is committed.

Use the clean CI-equivalent validation for a committed candidate:

```bash
make release-check-clean
```

## Approval and recovery

Configure any required GitHub Environment approval outside this repository. A `release` Environment can protect the release-creation job, and a separate `crates-publish` Environment can protect the irreversible publication job.

Before crates.io publication, cancel a pending run or delete a mistaken GitHub Release only after confirming the immutable tag must remain for auditability. Never move or reuse that tag. After crates.io publication, use crates.io yanking where appropriate, mark the GitHub Release as withdrawn or superseded, and publish a corrected higher version.
