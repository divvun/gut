# Changelog

## [0.8.0] - 2026-02-12

### Added

- **`gut label` commands** for managing GitHub labels in bulk across repositories (#218):
  - `gut label list` -- lists all labels in a table with color swatches
  - `gut label create` -- creates a label across matching repos
  - `gut label delete` -- deletes a label across matching repos
  - `gut label rename` -- renames a label (and optionally updates color/description) across matching repos
- **`gut rename team <SLUG> <NEW_NAME>` command**: Renames a GitHub organisation team with a confirmation prompt (#215)
- **`gut show repository <REPO_NAME>` command**: Displays all teams and collaborators for a repository with permission levels and affiliation (org/direct/outside) (#216)
- **`gut show teams --tree` flag**: Renders teams as a hierarchical tree showing parent/child relationships (#214)
- **`gut show team` parent/child info**: Now displays parent team and child teams in the header (#214)
- **Git LFS support** in `gut clone` and `gut pull`: Automatically detects LFS usage via `.gitattributes` and pulls LFS objects, with status shown in the summary table (#221)
- **`--json` flag for `gut topic list`**: Output topic results as JSON (#220)

### Changed

- **`gut topic get` renamed to `gut topic list`**: Results now displayed in a formatted table instead of plain text (#220)
- **`--organisation` now optional** in `add repos`, `add users`, `create discussion`, `create team`, `invite users`, `remove users`, `set team permission`, `show access`, and `show members` -- falls back to the default owner from configuration (#217)
- **`show access` and `show repo`**: `--organisation` flag renamed to `--owner` to reflect support for both organisations and personal accounts (#217)
- **Unified 404 error handling** with helpful guidance across team/org commands (#217)
- **Dependency updates**

### Fixed

- Corrected success message grammar from "There is no error!" to "There were no errors!" across multiple commands

## [0.7.0] - 2026-02-05

### Added

- **`gut show teams` command**: Lists all teams in an organisation with slug, name, and description
- **`gut show team <slug>` command**: Shows detailed information about a specific team including:
  - Team members with their roles (member/maintainer)
  - Repositories accessible by the team with permission levels (admin/maintain/write/triage/read)
  - Color-coded output for roles and permissions
- **`gut show access` command**: Shows user access levels across repositories in a compact table format (#207)
  - Single line per repo with columns for each user's access level
  - `--long` (`-l`) flag for detailed per-user tables
  - Color-coded permission levels
- **`gut show members` command**: Renamed from `show users`, shows organisation members with permissions and 2FA status (#207)
- **Static Linux musl builds**: Added configuration for fully static Linux binaries (#210)

### Changed

- **`show users` renamed to `show members`**: Better reflects that it shows organisation membership (#207)

### Fixed

- **`clone` command**: Now filters out repos that don't belong to the current owner (#206)
- **`show users` command**: Fixed broken command that required unnecessary `user:email` permission (#207)
- **Local path error handling**: Improved error messages for path-related issues (#47)

## [0.6.0] - 2026-01-31

### Added

- **`gut health` command**: Comprehensive health check for repositories and system configuration
  - **NFD/NFC normalization detection**: Finds filenames stored in decomposed Unicode form that have precomposed equivalents, which cause conflicts on macOS
  - **Case-duplicate detection**: Finds files differing only in letter case (e.g., `File.txt` vs `file.txt`) that cause issues on case-insensitive filesystems
  - **Large file detection**: Identifies files exceeding threshold (default 50 MB) that should be tracked by Git LFS, with separate handling for files matching `.gitignore`
  - **Long path detection**: Warns about filenames and paths exceeding length limits for Windows compatibility
  - **System configuration checks**: Git version (min 1.7.10), `core.precomposeUnicode` (macOS), `core.autocrlf` (Unix), Git LFS installation
  - Per-owner summaries with repo counts and issue breakdowns
  - Actionable recommendations for fixing each issue type
  - Configurable thresholds: `--large-file-mb`, `--filename-length-bytes`, `--path-length-bytes`
  - Single owner (`-o`) or all owners (`-a`) mode

### Fixed

- **`status` command**: Now properly handles NFD-encoded filenames in git status output

## [0.5.0] - 2026-01-21

### Added

- **Personal account support**: gut now works with both GitHub organizations and personal user accounts as owners (#200)
- **`show repos` improvements** (#201):
  - `--json` flag for structured JSON output
  - `--default-branch` flag to show default branch column (requires extra API calls)
  - Clean table format showing repository name and clone status by default
- **Spinner for all GitHub API queries**: All commands that query GitHub now show "Querying GitHub for {owner} repositories..." spinner

### Changed

- **Terminology**: "organisation" → "owner" throughout CLI flags, help text, and output to reflect support for both orgs and users
  - `--organisation` flag renamed to `--owner` (with `--organisation` kept as alias)
  - `set default organisation` → `set default owner`
  - `--all-orgs` → `--all-owners`
- **`status` command**: Failed repos now shown in main table with dashes instead of separate error section
- **Consolidated spinner code**: Spinners moved into shared query functions, reducing duplication

### Fixed

- **`set secret` command**: Fixed panic when running the command
- **`status` command**: Now handles per-repo errors gracefully instead of failing entirely

## [0.4.0] - 2025-01-19

### Added

- **Progress bars** for network commands: `clone`, `fetch`, `pull`, `push`, `commit`, `template apply`
- **Spinner** shown while querying GitHub API ("Querying GitHub for matching repositories...")
- **`--fetch` option** for `gut status` to fetch before checking status

### Changed

- **`commit` & `push`**: No longer query GitHub API unless `--topic` filter is used — uses local directories instead (much faster)
- **`fetch`**: Now parallelized, making it significantly faster
- **Logging**: Changed default log level from Debug to Warn (use `RUST_LOG=debug` for verbose output)
- **Help text**: Permission values and list syntax now shown on first line (visible with `-h`)
  - e.g., `Permission (pull | push | admin | maintain | triage)`
  - e.g., `Usernames to add (eg: -u user1 -u user2)`

### Fixed

- Added `parse_permission` validator to `set permission` command (was missing validation)
- Fixed various cargo clippy warnings

### Removed

- **`--use-https` flag** removed from `commit` and `push` commands (was unused since these commands only operate on local repos)

## [0.3.0] - 2025-01-15

### Added

- **`--all-orgs` (`-a`) flag** for multi-organization support with summary reports
- **`--version` (`-v`) flag** to display version information
- **`template refresh` command** to update placeholder substitutions in existing repos
- **Parallelization** of many commands using rayon for improved performance

### Fixed

- Fixed `template apply --continue` failing if patch failed to apply (#193)
- Fixed malformed patch format by ensuring correct newline handling (#195)
- Fixed default root path (now correctly `~/gut`)
- Repos with only new files now considered clean (can be pulled)

## [0.2.x] and earlier

See git history for changes prior to 0.3.0.
