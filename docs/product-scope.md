# Product scope

## Purpose

`cocommit` is a small Rust terminal UI for creating Conventional Commits. It guides the user through the commit header fields and continuously previews the final message before executing `git commit`.

The sole v1 workflow is:

1. Run `cocommit` inside a Git working tree with staged changes.
2. Complete the commit fields using the keyboard.
3. Review the live preview.
4. Select Commit.
5. Let the installed Git CLI create the commit.

The supported non-interactive commands are `cocommit --help` and `cocommit --version`. All other arguments are rejected; field prefills and automation remain out of scope.

## Commit header

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

## Required behavior

- Provide the standard Conventional Commit types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, and `revert`.
- Permit arbitrary custom types.
- Support an optional free-text scope.
- Support a breaking-change marker.
- Require a one-line description.
- Accept an optional numeric issue or PR identifier and render it as `(#<issue>)`.
- Provide a live preview for every draft change.
- Support an explicit `-S` Git signing choice.
- Refuse to open the form outside a usable non-bare Git working tree or when there are no staged changes.
- Refuse to open the form when standard input or standard output is not an interactive terminal.
- Show Git's own output when committing succeeds or fails.

## Non-goals for v1

- Staging or unstaging files.
- Diff viewing, history, branches, pushing, or pull requests.
- Commit body, footers, amend mode, empty commits, or changelog generation.
- AI-generated messages.
- Replacing Git hooks, credentials, signing, or editor workflows.
- Repository-local configuration.
- Configurable types, scopes, or issue formatting.
- Dry-run, copy-only mode, or CLI prefill arguments.

These ideas may be reconsidered only after the focused commit-header workflow is stable.
