# Compatibility and deprecation

## Public contracts

The `0.1.x` public contracts are the `cocommit` executable, `--help`,
`--version`, documented exit codes, the schema-versioned global configuration,
and `.cocommit.toml` repository message policy. The interactive layout and
keyboard behavior are documented behavior, but terminal key encodings can vary
by terminal; known limitations appear in [Terminal support](terminal-support.md).

Patch releases do not intentionally change a documented command, exit code,
configuration meaning, canonical message rendering, or supported archive target.
Minor releases may add optional configuration fields or controls while retaining
valid existing configuration and defaults. Breaking changes require a major
version after `0.1.0`.

## Configuration evolution

New configuration requires an explicit `schema_version`. Unsupported, future,
mixed, and unknown configuration is rejected with its path rather than guessed.
Configuration migrations are documented before release; cocommit never rewrites
user or repository configuration automatically. A deprecated setting remains
documented for at least one minor release with its replacement and removal
version.

## Release notes

Every version-bump pull request updates the `Unreleased` section in
[CHANGELOG.md](../CHANGELOG.md). The approved entry becomes the GitHub Release
notes for the immutable version tag. Corrections are published as a higher
version; published versions, tags, and release assets are never replaced.
