# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2026-10-25

### Added

- Displayed the running cocommit version in the terminal UI.

### Changed

- Shared common UI layout rendering helpers.

### Fixed

- Used the version placeholder supported by the automated bump action.

## [0.2.2] - 2026-10-14

### Fixed

- Added a portable commit submission shortcut across supported terminals.

## [0.2.1] - 2026-09-17

### Fixed

- Added word-based cursor navigation in the commit composer.

## [0.2.0] - 2026-09-16

### Added

- Made commit header fields configurable.
- Resolved composer defaults from configured commands.

### Changed

- Adapted the composer layout and accent styles to its configured fields.

### Fixed

- Authenticated Git before the release workflow pushes a tag.

## [0.1.0] - 2026-09-08

### Added

- Added a keyboard-driven terminal UI for composing validated Conventional Commits.
- Supported multiline bodies, footers, commit signing, configurable sections, and repository-specific message policy.
- Added Git preflight checks, staged-change inspection, staged-file exclusion, and commit execution from the working directory.
- Added global and repository-local configuration.
- Added terminal restoration, bounded input, portable integration tests, and release automation.

[Unreleased]: https://github.com/jfrz38/cocommit/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/jfrz38/cocommit/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/jfrz38/cocommit/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/jfrz38/cocommit/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/jfrz38/cocommit/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jfrz38/cocommit/releases/tag/v0.1.0
