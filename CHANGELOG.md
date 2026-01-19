# Changelog

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
