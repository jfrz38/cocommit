# Roadmap

## Purpose

This roadmap describes the work required to turn the current private pre-release build into a production-ready and complete Conventional Commit composer. Iteration numbers continue the historical implementation plan, which ends at Phase 9.

The roadmap is intentionally split into two horizons:

1. Work required before the first public `0.1.0` release.
2. Enhancements that can be delivered after `0.1.0` without weakening the initial release.

No tag, GitHub Release, or public package should be created until Iterations 10 through 23 and the final release gate are complete. Repository visibility is changed only in the final pre-release iteration.

## Product boundary

`cocommit` should become a complete commit composer, not a general-purpose Git client.

It owns:

- Composing and validating complete Conventional Commit messages.
- Presenting context about the prepared Git index and removing an accidental staged file.
- Applying user and repository message conventions.
- Safely handing the validated message to the installed Git CLI.

Git continues to own:

- Staging files.
- Full diff and history navigation.
- Branches, remotes, pushes, and pull requests.
- Identity, credentials, hooks, editors, and signing infrastructure.

AI-generated messages, remote hosting integrations, and package-manager distribution are optional enhancements. They are not allowed to delay the first release unless the product boundary changes explicitly.

## First-release scope

The first `0.1.0` release requires both production readiness and the complete core workflow.

Production readiness means:

- Releases cannot happen accidentally.
- Terminal state is restored on all supported termination paths.
- Errors are actionable and command-line behavior is predictable.
- Supported platforms run tests and produce verified artifacts.
- Dependencies and release automation have explicit security controls.
- Installation, support, and recovery procedures are documented and rehearsed.

The complete core workflow means:

- Header, body, and footers can form one canonical Conventional Commit message.
- Repository-local conventions can be configured and validated.
- The user can review a concise summary of staged changes.
- Git remains the final authority and executes the commit with its normal environment.

CLI prefills, copy-only operation, draft recovery, amend mode, and external integrations are useful but are not part of the `0.1.0` release gate.

## Delivery rules

Every iteration must follow these rules:

- Record new product or architectural decisions before implementation when the choice affects future compatibility.
- Keep message formatting and validation independent from terminal rendering.
- Add automated tests for stable behavior introduced by the iteration.
- Update user and technical documentation in the same change as the behavior.
- Keep `make check` passing.
- Do not weaken Git hooks, signing, credentials, or native error output.
- Do not add a second message-formatting path for CLI or TUI entry points.
- Do not mark an iteration complete while its required manual checks are undocumented or outstanding.

## Pre-release iterations

### Iteration 10: Release control

The current release workflow reacts to pushes to `main`. This remains the approved trigger: a reviewed version bump merged to `main` is the explicit request for a GitHub Release. The workflow must still be safe to rehearse and must not let later merges rewrite an existing release.

Work:

- Keep automatic GitHub Release creation for a new package version merged to `main` and make later merges with that version no-ops.
- Add a manual, non-publishing rehearsal for the exact commit selected by the dispatch ref.
- Separate unprivileged validation from tag creation and package publication.
- Grant `contents: write` only to the job that creates the final tag and GitHub Release.
- Grant `id-token: write` only to the job that authenticates with crates.io.
- Add a non-publishing rehearsal path that builds and validates the candidate without creating a tag or release.
- Require a clean checkout for CI release validation instead of relying on `--allow-dirty`.
- Make retries idempotent and fail if an existing tag points to another commit.
- Document the approval and rollback process.

Completion criteria:

- A push or merge to `main` creates a release only when its package version has no immutable tag and GitHub Release.
- The complete validation path can run in the private repository without publishing anything.
- Privileged jobs do not compile or execute unnecessary project or dependency code.
- Duplicate and conflicting release attempts have automated tests or documented rehearsal evidence.

### Iteration 11: CLI contract and diagnostics

The binary needs a small, stable command-line contract even though its primary workflow remains interactive.

Work:

- Add `-h` and `--help` without requiring a Git repository or interactive terminal.
- Add `-V` and `--version` from package metadata.
- Reject unknown arguments instead of silently opening the TUI.
- Define exit behavior for success, cancellation, usage errors, preflight failures, terminal failures, and Git commit failures.
- Check that the required standard streams are interactive terminals before initializing raw mode.
- Distinguish a missing Git executable from repository, permission, `safe.directory`, and corrupt-repository failures.
- Preserve useful Git status and stderr context during preflight.
- Add a concise troubleshooting section for common startup failures.

Completion criteria:

- Help and version work outside a repository and through redirected output.
- Unknown arguments produce a usage error and do not initialize the terminal.
- Non-interactive invocation fails before emitting terminal control sequences.
- Git preflight errors identify the failed command and retain actionable context.
- CLI behavior has unit or integration coverage.

### Iteration 12: Terminal and input hardening

The normal cleanup path is already defensive. This iteration covers abnormal process and input paths expected from a production terminal application.

Work:

- Restore raw mode, alternate screen, bracketed paste, and cursor state after supported termination signals.
- Install a panic restoration strategy that does not hide the original panic.
- Decide and document suspension and resumption behavior on Unix.
- Preserve normal `Ctrl+C` cancellation through the event loop.
- Reject NUL, escape, and other unsupported control characters from typed and pasted input.
- Define generous but finite limits for every text field and paste operation.
- Make rendering and cursor calculations safe for large Unicode input.
- Verify cleanup remains idempotent when initialization only partially succeeds.

Completion criteria:

- Automated tests cover input sanitization and size boundaries.
- PTY tests cover normal exit, cancellation, initialization failure, and at least one abnormal termination path where supported.
- Manual checks confirm that supported terminals remain usable after forced termination.
- Oversized or invalid pasted input produces feedback without panicking or corrupting the terminal.

### Iteration 13: Staged-change context

Users should be able to confirm the scope of the prepared commit without turning `cocommit` into a staging or diff client.

Work:

- Read a summary of the Git index using explicit Git argument arrays.
- Show the total number of staged files.
- Distinguish additions, modifications, deletions, and renames where Git provides them.
- Show concise insertion and deletion statistics.
- Present a bounded, scrollable file summary in the TUI.
- Handle subdirectories, linked worktrees, Unicode names, and unusual file names safely.
- Refresh or clearly mark the summary if the index changes while the form is open.
- Keep index interaction limited to unstaging an explicitly selected file; do not add staging or diff rendering.

Completion criteria:

- The summary matches Git for representative temporary repositories.
- Empty-index behavior remains a preflight failure.
- No shell parsing or lossy file-name splitting is introduced.
- The compact layout remains usable when the summary contains many files.
- Staging and full diff rendering remain outside the product boundary.

### Iteration 14: Repository-local configuration

Teams need versioned conventions while individual UI preferences remain global. This iteration requires an ADR before implementation.

Work:

- Choose and document the repository-local configuration path, provisionally `.cocommit.toml` at the work-tree root.
- Resolve the repository root through Git so invocation from subdirectories and linked worktrees is consistent.
- Define precedence as repository configuration over global configuration over built-in defaults.
- Reserve a documented future position for CLI overrides without adding prefills yet.
- Add configurable commit types and scope suggestions.
- Add subject length, capitalization, and punctuation policies.
- Add configurable issue identifiers and rendering formats.
- Keep unknown-key rejection and contextual file errors.
- Define schema compatibility and migration rules before a public format is released.
- Separate UI preferences from repository message policy in the model and documentation.

Completion criteria:

- Configuration precedence has exhaustive tests.
- Missing files use defaults, while invalid existing files fail with their path and source.
- A repository can version conventions without changing user-global preferences.
- Existing `sign` behavior remains compatible and clearly documented.
- The configuration schema is documented with complete examples.

### Iteration 15: Complete commit-message domain

Status: complete.

The domain model must represent the complete message before the TUI attempts to edit it.

Work:

- Extend the commit model with an optional body and ordered footers or trailers.
- Define one canonical renderer for header, body, and footer separation.
- Support `BREAKING CHANGE:` and `BREAKING-CHANGE:` semantics consistently with the Conventional Commits specification.
- Support common trailers such as `Closes`, `Refs`, and `Co-authored-by` without hard-coding a closed list.
- Define validation for blank lines, footer tokens, multiline values, and duplicate semantic fields.
- Decide how the header breaking marker and breaking footer interact.
- Pass the complete message to Git as data without shell interpretation.
- Preserve Git's standard streams for hooks, signing, and pinentry.

Completion criteria:

- Table-driven tests cover header-only, body, footer, and breaking-change combinations.
- Preview and commit execution use the same renderer.
- Message whitespace is intentional, deterministic, and asserted exactly.
- Messages containing quotes, Unicode, and multiline content reach Git unchanged.
- No temporary commit-message file or alternate transport is introduced without an explicit security and cleanup decision.

### Iteration 16: Multiline and footer TUI

Status: implementation complete; manual visual check pending.

After the domain is stable, the terminal interface can expose complete message editing.

Work:

- Add multiline body editing with predictable newline behavior.
- Add structured creation, editing, ordering, and removal of footers.
- Provide an efficient path for adding a breaking-change description.
- Add scrolling and focus navigation across header, body, footers, staged context, preview, and submit.
- Show a subject length indicator and active repository policy.
- Keep validation errors field-specific and move focus to the first invalid value.
- Preserve form state when help or the type picker opens.
- Adapt full, compact, and too-small layouts.
- Review focus visibility, contrast, and keyboard discoverability.

Completion criteria:

- Every complete domain message can be created and edited through the TUI.
- State-transition tests cover multiline and footer operations.
- Render smoke tests cover long bodies, many footers, compact terminals, and wide Unicode text.
- The interface remains fully usable without a mouse.
- Preview output exactly matches the message later passed to Git.

### Iteration 17: Configurable TUI sections

The complete composer should remain compact for users who do not need every optional workflow. Visibility is a personal global preference, never a repository convention: hiding an editor must not silently change the repository's message policy.

Work:

- Add global `ui.sections` preferences for `staged_changes`, `body`, `footers`, and `issue`.
- Default every configurable section to visible so an absent configuration preserves the current interface exactly.
- Keep Type, Scope, Breaking, Subject, Sign commit, Preview, and Commit visible in every configuration.
- Derive focus order, compact and expanded layouts, help, keyboard hints, and validation routing from the visible sections.
- Ensure hidden Body, Footers, and Issue start empty, have no editor or shortcut, and cannot receive a validation error or focus.
- Keep Git preflight mandatory when Staged changes is hidden, but remove file inclusion choices and commit every file that remains staged at submission.
- Keep `.cocommit.toml` limited to `[message]`; repository configuration must reject `[ui]` and cannot hide a user's controls.

Completion criteria:

- No configuration and all-true preferences reproduce the current layout and navigation.
- Every combination of the four section preferences has a complete keyboard focus path to Preview and Commit.
- Hidden sections consume no layout space and expose no row, help entry, key hint, modal, or inactive action.
- Hidden staged context never constructs an exclusion or unstage operation, while preflight still rejects an empty index.
- Configuration, application-state, render-smoke, Git-integration, and manual-terminal tests cover the default and hidden-section flows.

### Iteration 18: Repository policy enforcement

Status: implemented.

Configuration becomes valuable when the form can guide users toward the repository's actual conventions before a hook rejects the commit.

Work:

- Apply configured type, scope, subject, and issue policies during editing and submission.
- When a configured policy requires an optional message part, expose its editor or report an actionable configuration conflict rather than making compliance impossible.
- Distinguish required values, allowed values, and non-blocking suggestions.
- Show active constraints without overcrowding the form.
- Add a documented, deliberately limited compatibility strategy for common commitlint rules.
- Do not execute arbitrary repository code as configuration.
- Do not attempt to reproduce every hook or its runtime environment.
- Continue treating Git and its hooks as the final authority.

Completion criteria:

- Policy failures are reported before the terminal closes.
- Configured suggestions improve entry without becoming unintended hard restrictions.
- Unsupported commitlint rules are explicit rather than silently misinterpreted.
- Hooks still run normally and their output remains native Git output.
- Policy behavior is covered by domain and application-state tests.

### Iteration 19: Cross-platform quality

Status: complete.

Compilation alone is insufficient for a cross-platform TUI. Every advertised platform must execute meaningful tests and produce the binary that will be distributed.

Work:

- Run formatting, linting, tests, and builds on Linux.
- Run tests and release builds, not only `cargo check`, on Windows and macOS.
- Require Git explicitly in CI so the real integration test cannot silently disappear.
- Add real-Git integration coverage for preflight, subdirectories, bare repositories, no staged changes, and linked worktrees where available.
- Add PTY lifecycle coverage on platforms where CI supports it.
- Validate workflow files with `actionlint`.
- Validate shell fragments with ShellCheck.
- Define and execute a manual terminal matrix covering the supported Windows, macOS, Linux, SSH, and multiplexer environments.
- Record known keyboard limitations such as terminals that cannot distinguish `Ctrl+Enter`.

Completion criteria:

- All advertised operating systems execute automated tests and build release binaries.
- The Git integration suite cannot report success merely because Git is absent in CI.
- PTY and manual smoke-test results are recorded for the release candidate.
- The supported terminal matrix and known limitations are public documentation.

### Iteration 20: Supply-chain hardening

Status: implementation complete; external GitHub controls and rehearsal evidence pending.

Release automation must minimize the authority granted to dependencies, third-party Actions, and generated artifacts.

Work:

- Pin every third-party GitHub Action to a reviewed full commit SHA.
- Configure Dependabot or Renovate to propose controlled Action and Cargo updates.
- Add `cargo audit` or `cargo deny` checks for security advisories.
- Define and enforce an allowed dependency-license policy.
- Protect release tags through repository rulesets.
- Protect crates.io publication through a GitHub Environment with required approval.
- Generate an SBOM for release artifacts.
- Generate checksums and provenance attestations for distributed binaries.
- Document the response process for a vulnerable dependency or compromised release.

Completion criteria:

- CI fails on disallowed advisories or licenses according to the documented policy.
- Mutable Action tags are not used in privileged workflows.
- Release tags cannot be moved or deleted through the normal maintainer workflow.
- Artifacts can be matched to checksums and their source workflow.
- A supply-chain incident has a documented revocation and replacement procedure.

### Iteration 21: Distribution artifacts

Status: in progress.

`cargo install` remains the baseline installation path, but users should not need a Rust toolchain when verified binaries can be provided safely.

Work:

- Validate the crates.io package from a clean checkout and inspect its exact contents.
- Build release binaries for the explicitly supported Linux, macOS, and Windows targets.
- Package binaries with license, version, target name, and minimal installation instructions.
- Attach checksums, SBOM, and provenance produced by the release workflow.
- Decide and document the code-signing status for Windows and macOS binaries.
- Test `cargo install --path` and packaged binaries in clean environments.
- Prepare package-manager integration as post-release work rather than blocking `0.1.0`.

Completion criteria:

- Every advertised artifact runs on its target and reports the expected version.
- The crates.io dry run and artifact build use the exact candidate commit.
- Archive contents, names, checksums, and provenance are deterministic and documented.
- Unsupported targets and unsigned-binary warnings are explicit.

### Iteration 22: Documentation and governance

An external user must be able to install, configure, operate, diagnose, and report problems without internal project knowledge.

Work:

- Update the root README for the final feature and installation scope.
- Document global and repository-local configuration with precedence examples.
- Document complete-message composition, staged context, and repository policy behavior.
- Add troubleshooting for Git, hooks, signing, terminal state, and unsupported terminals.
- Add `SECURITY.md` with private vulnerability reporting instructions and supported versions.
- Add `CONTRIBUTING.md` with local setup, quality checks, architecture boundaries, and pull-request expectations.
- Define compatibility and deprecation policies for CLI and configuration changes.
- Create the initial changelog and release-note process.
- Document release failure, artifact revocation, and replacement procedures.
- Verify every command, badge, link, and example from a clean user perspective.

Completion criteria:

- Documentation matches the release candidate rather than planned behavior.
- Security and contribution channels are usable before repository visibility changes.
- Installation and troubleshooting instructions have been followed successfully from a clean environment.
- No documentation presents an unpublished package as currently available.

### Iteration 23: Private release rehearsal

The complete release path must be exercised in the private repository without producing the first public release.

Work:

- Select one clean commit as the release candidate.
- Review the structure of `src/app.rs` and `src/ui.rs` before freezing the candidate, dividing them at natural responsibility boundaries when their size or cohesion impedes review.
- Keep that modularization behavior-preserving: separate input editing, type selection, preview and scrolling, footers, staged changes, layouts, overlays, and viewports without adding unnecessary abstractions or weakening `ui.sections` navigation.
- Move or reorganize the affected tests with their modules, then run `make check` and repeat the default, hidden-section, footer, preview, staged-change, and compact-terminal smoke tests.
- Review the complete candidate diff for dead code, duplication, accidental artifacts, and regressions; resolve every finding before the candidate is frozen.
- Run the complete unprivileged quality and package validation jobs.
- Produce every final artifact without creating a public tag, GitHub Release, or crates.io version.
- Install and smoke-test the candidate on every supported platform.
- Exercise cancellation, signing, hook rejection, non-repository startup, no staged changes, paste handling, and forced termination.
- Review checksums, SBOM, provenance, archive contents, and version metadata.
- Audit tracked history and repository configuration for credentials, local paths, crash dumps, and accidental artifacts.
- Resolve every open release-gate item and repeat affected checks.

Completion criteria:

- The rehearsal uses the same build and validation definitions as the real release.
- All automatic and manual evidence is attached to or linked from the release-candidate record.
- No critical or high-risk release issue remains open.
- The structural review records whether `src/app.rs` and `src/ui.rs` required division, and any resulting modules have clear, documented responsibilities without reducing coverage or changing behavior.
- The candidate can be published without changing source, dependencies, or workflow definitions.

### Iteration 24: Public launch and first release

This is the final pre-release iteration. Repository visibility changes only after the private rehearsal succeeds.

Work:

- Make the repository public.
- Verify branch protections, tag rulesets, workflow permissions, and the publication Environment.
- Verify public issue, contribution, and security-reporting paths.
- Create `v0.1.0` at the rehearsed commit.
- Create the first GitHub Release with the rehearsed artifacts and release notes.
- Perform the initial crates.io publication using the documented short-lived credential procedure.
- Verify `cargo install cocommit --locked` from a clean environment.
- Verify every public archive, checksum, SBOM, attestation, badge, and documentation link.
- Configure crates.io Trusted Publishing for all subsequent versions.
- Revoke the initial publication token immediately after success.

Completion criteria:

- The public tag, GitHub Release, package version, source commit, and artifacts all match.
- Installation succeeds through every advertised method.
- No repository secret or private-only reference became public.
- Trusted Publishing is configured and a future publication rehearsal can authenticate without a long-lived token.
- The first release is announced only after verification is complete.

## Release gate for 0.1.0

Iteration 24 cannot start until all of the following are true:

- Iterations 10 through 23 are complete.
- `make check` passes from the clean candidate checkout.
- Security, license, workflow, package, and artifact checks pass.
- Linux, macOS, and Windows release candidates pass their defined automated and manual tests.
- Header, body, footer, repository policy, and staged-context workflows pass acceptance tests.
- Terminal restoration has been verified for normal, cancelled, error, panic, and supported signal paths.
- Documentation has been tested as an external user would follow it.
- There are no unresolved critical or high-severity defects.
- The release candidate is immutable: publishing requires no source or workflow changes.
- A maintainer explicitly approves changing repository visibility and publishing `0.1.0`.

## Post-0.1.0 roadmap

The following work improves integration and efficiency but is not required for a safe and complete first release. Its order should be reconsidered using real user feedback.

### Iteration 25: Working-tree context and optional index preparation

The default workflow remains committing from the prepared Git index. This iteration may expose broader working-tree context, but it must not make a file-level toggle imply an unsafe or ambiguous Git operation.

Work:

- Record an ADR that defines whether optional index preparation remains within the product boundary and how it coexists with the staged-only composer.
- Keep `staged` as the default view, showing exactly the content eligible for the commit.
- Add an optional global UI preference for the initial changes view: `staged` or `all`. Hiding staged context remains exclusively `ui.sections.staged_changes = false` and must not change which files are committed implicitly.
- In the `all` view, present separate staged, unstaged, untracked, and partially staged sections with unambiguous labels.
- Keep inclusion checkboxes exclusive to the staged commit snapshot. Use explicit actions such as `Stage file` for changes outside the index.
- Decide whether full-file staging is supported at all. If it is, warn before staging a partially staged file and never silently add its unstaged content to the reviewed commit snapshot.
- Do not claim hunk-level selection unless the application can faithfully display, select, stage, and test hunks.
- Define refresh behavior for external index or working-tree changes while the form is open.
- Preserve cancellation without changing the index. Apply any selected index mutations only after terminal restoration and before the final Git commit.

Completion criteria:

- The staged-only default remains usable without configuring the new view.
- Every rendered item clearly identifies whether it is staged, unstaged, untracked, or partially staged.
- A file-level action never causes unstaged content from a partially staged file to enter the commit without explicit confirmation and a refreshed review state.
- Temporary-repository integration tests cover staged, unstaged, untracked, partially staged, renamed, deleted, and unusual-path cases.
- State-transition and render tests cover switching views, explicit index actions, cancellation, and external-change feedback.
- The updated product boundary, configuration schema, interaction model, Git command behavior, and recovery semantics are documented in the same change.

### CLI automation

- Prefill fields with `--type`, `--scope`, `--issue`, and `--breaking`.
- Add `--print`, copy-only, and dry-run modes that reuse the same domain validation and renderer.
- Add shell completions and a documented Git alias.
- Define a stable machine-readable output only if automation needs justify it.

### Workflow recovery

- Persist an opt-in local draft before invoking Git.
- Restore a draft after hook or signing failure.
- Allow retry without re-entering a complete message.
- Define privacy, cleanup, expiry, and repository-isolation rules for stored drafts.

### Contextual suggestions

- Infer issue identifiers from branch names through configurable patterns.
- Remember recent types and scopes locally when explicitly enabled.
- Read compatible `.gitmessage` templates.
- Add deeper commitlint compatibility only for rules that can be represented faithfully.

### History editing

- Add explicit amend mode.
- Load and edit the previous commit message.
- Clearly communicate history-rewriting behavior before execution.
- Keep log browsing and general history management outside the product boundary.

### Additional distribution

- Publish Homebrew, Scoop, and Winget packages.
- Add package-manager update automation after the core release process proves stable.
- Evaluate operating-system code signing when adoption and key-management requirements justify it.

### Optional remote or AI integrations

- Evaluate passive Jira or GitHub issue suggestions without taking ownership of credentials or remote workflows.
- Evaluate AI-assisted drafting only as an explicit, optional provider boundary with clear privacy behavior.
- Keep deterministic manual composition available without network access.
- Do not add push, pull-request creation, branch management, or a general Git hosting client.

## Roadmap maintenance

This document describes intended outcomes, not completed behavior. When an iteration starts, its decisions should be refined in the relevant product, architecture, interaction, testing, configuration, or release document. When it finishes, update the project status in [README.md](README.md), record any accepted scope changes here, and keep the implementation contract synchronized with the code.
