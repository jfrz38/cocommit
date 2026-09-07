# Configuration

`cocommit` reads optional TOML configuration from two places:

| Scope | Location | Purpose |
| --- | --- | --- |
| Global | Linux: `$XDG_CONFIG_HOME/cocommit/config.toml` or `~/.config/cocommit/config.toml`; macOS: `~/Library/Application Support/cocommit/config.toml`; Windows: `%APPDATA%\cocommit\config.toml` | UI preferences and default message policy |
| Repository | `.cocommit.toml` at the Git work-tree root | Message policy shared by the repository |

Built-in defaults are loaded first, followed by global configuration and then
repository configuration. Repository configuration cannot change UI settings.
Missing files are ignored. Invalid files are reported with their path and are
never rewritten by cocommit.

## Global preferences

The legacy form is supported for the signing default:

```toml
sign = false
```

New files use `schema_version = 1`:

```toml
schema_version = 1

[ui]
sign = true

[ui.sections]
staged_changes = true
body = true
footers = true
issue = true
```

All sections default to `true`. Type, Scope, Breaking, Subject, Sign commit,
Preview, and Commit always remain available. With `staged_changes = false`,
cocommit commits every staged file and does not offer file exclusion.

## Repository policy

Add `.cocommit.toml` to a repository to share commit conventions:

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

An explicit `types` list restricts commit types. Without one, the standard
types are suggestions and custom types are accepted. Scope suggestions are
never restrictive. `max_length` limits the subject; `capitalization` accepts
`allow`, `lowercase`, or `uppercase`; `terminal_punctuation` accepts `allow`,
`forbid`, or `require`. Issue identifiers are decimal numbers and are rendered
with the configured prefix and `parenthesized` or `plain` style.

Unknown keys, unsupported schema versions, and invalid values are rejected.
