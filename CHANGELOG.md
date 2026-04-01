# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `-v`/`--verbose` flag to show response body in error messages for debugging API/schema issues

### Changed
- Token storage now defaults to file-based (`~/.linear-cli/tokens.json`) instead of OS keychain, eliminating repeated password prompts on macOS. Use `lin login --keyring` to opt into keychain storage. Existing keychain tokens are read as a fallback.

## [0.6.0] - 2026-04-01

### Added
- `cycle create` command with `--starts`, `--ends`/`--duration`, `--name`, `--description` flags
- `cycle show` command with progress bar, status badge, and issues table
- `--cycle` flag on `issue create` and `issue edit` for assigning issues to cycles
- Cycle info displayed in `issue view` output
- Global `--json` flag for raw JSON output on all read commands (e.g., `lin --json project list`, `lin issue view ENG-1 --json`)
- `project view --content` flag to display the full project description/overview
- `project edit --content` flag to set the long-form project description/overview
- Pre-push git hook for running tests and changelog verification

### Fixed
- `login` command now respects the global `-w` workspace flag instead of always storing tokens under the `--name` default ("default")

## [0.5.0] - 2026-03-24

### Added
- Date filters for `issue list`: `--updated-since`, `--updated-before`, `--created-since`, `--created-before`, `--completed-since`, `--completed-before`, `--due-after`, `--due-before` with ISO 8601 and relative date support (e.g., `3d`, `1w`) (#15)
- Label filter for `issue list`: `--label` (repeatable for AND logic) (#16)
- Cycle filter for `issue list`: `--cycle` with name, number, or `current` shorthand (#16)
- Creator filter for `issue list`: `--creator` accepts name, email, `me`, or UUID (#16)
- Convenience filters for `issue list`: `--cancelled-since`, `--estimate`, `--estimate-gte`, `--estimate-lte`, `--parent`, `--no-parent`, `--has-children`, `--subscriber`, `--title` (#17)
- `issue attachments add` subcommand for attaching files to issues (#14)
- CHANGELOG.md with backfilled history, PR template, CI changelog check, and GitHub Release automation

### Fixed
- `issue edit --state` now correctly resolves state name to UUID instead of passing the name string (#12)

### Breaking Changes
- `issue attachments <id>` is now `issue attachments list <id>` — the `attachments` command is a subcommand group with `list` and `add` (#14)

## [0.4.0] - 2026-03-20

### Added
- `project update show` subcommand

## [0.3.0] - 2026-03-11

### Added
- `issue comment` subcommand as alias for `comment add` for agent discoverability (#10)

## [0.2.1] - 2026-03-10

### Fixed
- Missing user `id` fields in GraphQL queries (#4)
- File upload signed headers not being included from presigned URL (#7)

## [0.2.0] - 2026-03-04

### Added
- `cycle list`, `cycle show` commands
- `initiative list`, `initiative show` commands
- `issue list`, `issue me` commands
- Updated to current Linear API

## [0.1.0] - 2026-03-04

### Added
- Initial release
- Authentication with Linear API tokens
- Issue view, create, edit, search
- Project list, show, update
- Team list
- Comment view, add, edit
- Label list
