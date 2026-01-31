mod checks;
mod display;
mod types;

use super::common;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;
use types::{Issue, OwnerSummary, RepoCheckResult};

#[derive(Debug, Parser)]
/// Comprehensive health check for repositories
///
/// This command scans all files in local repositories and performs multiple checks:
///
/// FILE CONTENT CHECKS:
///
/// - NFD normalization issues: Filenames with decomposed Unicode characters (NFD)
///   that have a composed (NFC) equivalent, which can cause conflicts on macOS.
///   The command detects such files and lists them. The target is to have all filenames
///   encoded as NFC in the git repository index, and let the conversion to NFD happen
///   on checkout on macOS systems.
///
/// - Case-duplicate filenames: Files with identical names except for letter case
///   (e.g., File.txt and file.txt), which cause problems on case-insensitive
///   filesystems like macOS and Windows. Such filenames are reported to help users
///   identify and change filenames on a case-sensitive filesystem like Linux.
///
/// - Large files not tracked by LFS: Files exceeding size threshold (default 50 MB)
///   that should be tracked by Git LFS, with separate detection for files that
///   match .gitignore patterns (indicating they shouldn't be in Git at all)
///
/// - Long filenames and paths: Files with names or paths that may cause problems
///   on systems with path length limits (especially Windows with 260 char limit)
///
/// SYSTEM CONFIGURATION CHECKS:
///
/// - Git version (minimum 1.7.10 required)
///
/// - core.precomposeUnicode setting (macOS)
///
/// - core.autocrlf setting (Unix systems)
///
/// - Git LFS installation status
///
/// The command provides detailed recommendations for fixing any issues found.
pub struct RepoHealthArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
    #[arg(long, default_value = "50")]
    /// Size threshold in MB for detecting large files not tracked by LFS
    pub large_file_mb: u64,
    #[arg(long, default_value = "200")]
    /// Filename length threshold in bytes for warnings
    pub filename_length_bytes: usize,
    #[arg(long, default_value = "400")]
    /// Full path length threshold in bytes for warnings
    pub path_length_bytes: usize,
}

impl RepoHealthArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;

        let owners = if self.all_owners {
            common::get_all_owners()?
        } else {
            vec![common::owner(self.owner.as_deref())?]
        };

        let mut owner_summaries = Vec::new();

        for owner in &owners {
            let summary = self.check_owner(&root, owner)?;
            owner_summaries.push(summary);
        }

        // Print summaries
        if self.all_owners {
            // Multi-owner: print each owner's details, then final summary with recommendations
            for summary in &owner_summaries {
                display::print_owner_summary(self, summary, false);
            }
            display::print_final_summary(self, &owner_summaries);
        } else {
            // Single owner: print details with recommendations
            if let Some(summary) = owner_summaries.first() {
                display::print_owner_summary(self, summary, true);
            }
        }

        // Run system configuration health checks
        display::print_system_health_checks();

        Ok(())
    }

    fn check_owner(&self, root: &str, owner: &str) -> Result<OwnerSummary> {
        let owner_path = Path::new(root).join(owner);
        if !owner_path.exists() {
            return Ok(OwnerSummary {
                owner: owner.to_string(),
                total_repos: 0,
                issues: Vec::new(),
            });
        }

        let repos = std::fs::read_dir(&owner_path)
            .with_context(|| format!("Cannot read directory {:?}", owner_path))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect::<Vec<_>>();

        let total_repos = repos.len();

        // Process repos with progress bar
        let progress_message = format!("Checking {}", owner);
        let threshold_bytes = self.large_file_mb * 1024 * 1024;
        let filename_threshold = self.filename_length_bytes;
        let path_threshold = self.path_length_bytes;
        let results: Vec<RepoCheckResult> = common::process_with_progress(
            &progress_message,
            &repos,
            |repo_path| {
                checks::check_repo(
                    repo_path,
                    threshold_bytes,
                    filename_threshold,
                    path_threshold,
                )
            },
            |result| result.repo_name.clone(),
        );

        // Flatten all issues from all repos
        let issues: Vec<Issue> = results.into_iter().flat_map(|r| r.issues).collect();

        Ok(OwnerSummary {
            owner: owner.to_string(),
            total_repos,
            issues,
        })
    }
}
