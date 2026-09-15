# Configuration

`cocommit` reads optional TOML configuration from two places:

| Scope | Location | Purpose |
| --- | --- | --- |
| Global | Linux: `$XDG_CONFIG_HOME/cocommit/config.toml` or `~/.config/cocommit/config.toml`; macOS: `~/Library/Application Support/cocommit/config.toml`; Windows: `%APPDATA%\cocommit\config.toml` | UI preferences, default message policy, and executable defaults |
| Repository | `.cocommit.toml` at the Git work-tree root | Message policy shared by the repository |

Built-in defaults are loaded first, followed by global configuration and then
repository configuration. Repository configuration cannot change UI settings.
Global configuration may also define a default `[message]` policy; repository
configuration overrides message-policy values field by field.
Missing files are ignored. Invalid files are reported with their path and are
never rewritten by cocommit.

## Global preferences

Signing is enabled by default, and all configurable composer sections are visible by
default. Global preferences use `schema_version = 1`:

```toml
schema_version = 1

[ui]
sign = false
accent_color = "cyan"

[ui.sections]
type = true
scope = true
breaking = true
staged_changes = true
body = true
footers = true
issue = true
sign = true
```

All section flags default to `true`. Subject, Preview, and Commit always remain
available. Hidden fields do not contribute a value to the commit. The one
exception is `sign`: hiding its row preserves the effective `[ui].sign` value.
Setting `type = false` also hides Scope and Breaking and produces a subject-only
header, regardless of their flags. A `BREAKING CHANGE` footer remains available
when Footers is visible. With `staged_changes = false`, cocommit commits every
staged file and does not offer file exclusion. Hiding rows also lets the expanded
layout fit in shorter terminals.

`accent_color` colors the focused control and selected picker row. It accepts
`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`,
`dark_gray`, `light_red`, `light_green`, `light_yellow`, `light_blue`,
`light_magenta`, `light_cyan`, or `white`, and defaults to `cyan`.

## Executable defaults

The global file can prefill visible text fields from user-authored commands:

```toml
[defaults.type]
command = ["pwsh", "-NoProfile", "-File", "C:/Users/me/bin/commit-type.ps1"]

[defaults.scope]
command = ["git", "branch", "--show-current"]
```

`defaults.type`, `defaults.scope`, `defaults.body`, and `defaults.issue` are
supported. Each command is an executable followed by literal arguments; cocommit
does not invoke a shell, expand variables, or interpret the output. Scripts must
therefore name their interpreter explicitly where the platform requires it.

Commands run in the Git work-tree root before the terminal UI starts, only when
their field is visible. They receive closed stdin, inherit the user environment,
and have two seconds to finish and close their output streams. They must emit one
non-empty UTF-8 line within the field limit and an 8 KiB output limit. The value
is trimmed, validated against that field's format, and remains editable in the
composer. A missing executable, timeout, non-zero exit, or invalid output aborts
startup with an error. Repository files cannot define commands, so cloning or
opening a repository never opts into code execution. Commands should not start
background processes because the timeout only terminates the configured process,
not descendants that it starts.

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

[message.format]
separator = ":"
```

An explicit `types` list restricts commit types. Without one, the standard
types are suggestions and custom types are accepted. Scope suggestions are
never restrictive. `max_length` limits the subject; `capitalization` accepts
`allow`, `lowercase`, or `uppercase`; `terminal_punctuation` accepts `allow`,
`forbid`, or `require`. Issue identifiers are decimal numbers and are rendered
with the configured prefix and `parenthesized` or `plain` style.

`separator` controls the punctuation between the optional type header and the
subject. cocommit always appends one space: `":"` renders `feat: subject`, `"-"`
renders `feat- subject`, and `""` renders `feat subject`. It is omitted entirely
for subject-only messages. Separators other than `":"` may not comply with the
Conventional Commits specification.

Unknown keys, unsupported schema versions, and invalid values are rejected.
