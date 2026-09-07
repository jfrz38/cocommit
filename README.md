# 🥥 cocommit

**Craft complete Conventional Commit messages without leaving your terminal.**

[![CI](https://github.com/jfrz38/cocommit/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/jfrz38/cocommit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cocommit?logo=rust)](https://crates.io/crates/cocommit)
[![Downloads](https://img.shields.io/crates/d/cocommit)](https://crates.io/crates/cocommit)
[![License](https://img.shields.io/github/license/jfrz38/cocommit)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.94.1%2B-blue)](https://www.rust-lang.org)

Writing a good commit message should not interrupt your flow. `cocommit` guides
you through the type, scope, subject, body, footers, and staged files while
showing exactly what Git will receive.

The name combines **CO**nventional and **COMMIT**s, hence the coconut 🥥.

[example](https://github.com/jfrz38/cocommit/blob/main/docs/assets/example.mp4)

## Why cocommit?

- Build complete Conventional Commit messages without memorizing their syntax.
- See the final message while you write it, before it reaches Git.
- Catch an accidentally staged file and leave it out of the commit.
- Keep your hooks, signing, credentials, and usual Git output.
- Stay in the terminal from start to finish.

```text
feat(api)!: add authentication (#123)

Require an access token for protected routes.

BREAKING CHANGE: Unauthenticated requests now fail.
```

## Get started

Install from crates.io:

```sh
cargo install cocommit --locked
```

Then stage your work and open the composer from anywhere in that repository:

```sh
git add src/main.rs
cocommit
```

Start with the type, describe the change, and add the optional details that make
the message useful later. The preview is the exact message passed to Git.

## Keyboard first

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Move between fields |
| `Enter` | Advance, open a picker, edit a footer, or submit |
| `Ctrl+Enter` | Submit from any field |
| `Space` | Toggle Breaking, Sign, or a selected staged file |
| `Up` / `Down` | Navigate fields, lists, and the preview |
| `F1` | Show keyboard help |
| `Esc` / `Ctrl+C` | Cancel and restore the terminal |

## Make it yours

`cocommit` reads two optional TOML files:

- **Global** preferences live in your user configuration directory and set
  personal defaults for signing, visible composer sections, and optionally
  message policy.
- **Repository** policy lives in `.cocommit.toml` at the Git work-tree root.
  Commit this file to the repository so the whole team shares the same commit
  conventions.

Signing and all optional sections are enabled by default. Repository policy can
restrict commit types, offer scope suggestions, enforce subject rules, and set
the issue-reference format:

```toml
schema_version = 1

[message]
types = ["feat", "fix", "docs"]
scope_suggestions = ["api", "tui"]

[message.subject]
max_length = 72
capitalization = "lowercase"
terminal_punctuation = "forbid"

[message.issue]
prefix = "PROJ-"
style = "plain"
```

An explicit `types` list restricts the allowed types; omit it to keep the
standard types as suggestions while allowing custom types. Scope suggestions
are never restrictive.

Configuration is merged in this order: built-in defaults, global configuration,
then repository configuration. Each layer overrides the previous one field by
field. Repository files can change message policy but not personal UI settings.

See [configuration](docs/configuration.md) for exact paths, the full schema,
and more examples.

## Technical details

`cocommit` needs Git on `PATH`, staged changes, and an interactive terminal. It
does not stage files, amend commits, show diffs or history, manage branches, or
push. Git remains responsible for hooks, signing, credentials, editors, and
output.

`cocommit` is distributed as a Rust crate; prebuilt binaries are not available.
Rust 1.94.1 or newer is only required when installing from source.

## Development

```sh
git clone https://github.com/jfrz38/cocommit.git
cd cocommit
make run
make check
```

Use `make release-check` before a release. See
[Contributing](https://github.com/jfrz38/cocommit/blob/main/CONTRIBUTING.md),
[Security](https://github.com/jfrz38/cocommit/blob/main/SECURITY.md),
and [architecture](https://github.com/jfrz38/cocommit/blob/main/docs/architecture.md).
