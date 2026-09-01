# Product scope

## Purpose

`cocommit` is a small Rust terminal UI for creating Conventional Commits. It guides the user through the commit header fields and continuously previews the final message before executing `git commit`.

The sole v1 workflow is:

1. Run `cocommit` inside a Git working tree with staged changes.
2. Complete the commit fields using the keyboard.
3. Review the live preview.
4. When `ui.sections.staged_changes` is enabled, review the concise staged-change list and unstage accidental files if needed.
5. Select Commit.
6. Let the installed Git CLI create the commit.

The supported non-interactive commands are `cocommit --help` and `cocommit --version`. All other arguments are rejected; field prefills and automation remain out of scope.

## Commit message domain

The canonical rendered format is:

```text
<type>(<scope>)<breaking>: <message> (#<issue>)
```

Optional sections disappear completely when their values are absent.

```text
feat: add authentication
feat(api): add authentication
feat(api)!: change authentication API
feat(api): add authentication (#123)
```

The interface supports an optional multiline body and ordered footers, separated from the header and each other by one blank line. The live preview and final Git invocation use this same canonical message.

## Required behavior

- Provide the standard Conventional Commit types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, and `revert`.
- Permit arbitrary custom types.
- Support an optional free-text scope.
- Support a breaking-change marker.
- Require a one-line description.
- Accept an optional numeric issue or PR identifier and render it as `(#<issue>)`.
- Provide a live preview for every draft change.
- Show staged file count, additions, modifications, deletions, renames, and aggregate insertion/deletion statistics when Staged changes is enabled.
- Include or exclude the selected file with `Space`, preserving its working-tree content and keeping at least one file included for commit when Staged changes is enabled.
- Global visibility preferences control optional Body, Footers, Issue, and Staged changes sections; all four default to visible while Type, Scope, Breaking, Subject, Sign commit, Preview, and Commit remain visible.
- Hiding Staged changes still requires a non-empty Git index and commits all files that remain staged at submission.
- Support an explicit `-S` Git signing choice.
- Refuse to open the form outside a usable non-bare Git working tree or when there are no staged changes.
- Refuse to open the form when standard input or standard output is not an interactive terminal.
- Show Git's own output when committing succeeds or fails.

## Non-goals for v1

- Staging files.
- Full diff viewing, history, branches, pushing, or pull requests.
- Amend mode, empty commits, or changelog generation.
- AI-generated messages.
- Replacing Git hooks, credentials, signing, or editor workflows.
- Letting repository configuration control personal UI visibility. Repository policy and global UI preferences remain separate.
- Executing commitlint configuration, plugins, or arbitrary repository code. Supported static repository message policy is enforced as documented in ADR 0012.
- Dry-run, copy-only mode, or CLI prefill arguments.

These ideas may be reconsidered only after the focused commit-header workflow is stable.
