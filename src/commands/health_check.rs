use super::common;
use crate::git;
use crate::health;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use prettytable::{Table, cell, format, row};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

/// Width of separator lines in output
const LINE_WIDTH: usize = 80;

/// Bytes per megabyte for size conversions
const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

/// Maximum size in bytes for a Git LFS pointer file.
/// LFS pointer files are small text files that reference the actual content.
const LFS_POINTER_MAX_BYTES: usize = 200;

/// Magic prefix that identifies a Git LFS pointer file
const LFS_POINTER_PREFIX: &[u8] = b"version https://git-lfs.github.com/spec/";

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
pub struct HealthCheckArgs {
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

/// Lightweight tag for issue types - enables HashSet operations and exhaustive matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IssueKind {
    Nfd,
    CaseDuplicate,
    LargeFile,
    LargeIgnoredFile,
    LongPath,
}

/// Unified issue type - all issue data in enum variants
#[derive(Debug, Clone)]
enum Issue {
    Nfd {
        repo: String,
        file_path: String,
    },
    CaseDuplicate {
        repo: String,
        files: Vec<String>,
    },
    LargeFile {
        repo: String,
        file_path: String,
        size_bytes: u64,
    },
    LargeIgnoredFile {
        repo: String,
        file_path: String,
        size_bytes: u64,
    },
    LongPath {
        repo: String,
        file_path: String,
        path_bytes: usize,
        filename_bytes: usize,
    },
}

impl Issue {
    fn kind(&self) -> IssueKind {
        match self {
            Issue::Nfd { .. } => IssueKind::Nfd,
            Issue::CaseDuplicate { .. } => IssueKind::CaseDuplicate,
            Issue::LargeFile { .. } => IssueKind::LargeFile,
            Issue::LargeIgnoredFile { .. } => IssueKind::LargeIgnoredFile,
            Issue::LongPath { .. } => IssueKind::LongPath,
        }
    }

    fn repo(&self) -> &str {
        match self {
            Issue::Nfd { repo, .. } => repo,
            Issue::CaseDuplicate { repo, .. } => repo,
            Issue::LargeFile { repo, .. } => repo,
            Issue::LargeIgnoredFile { repo, .. } => repo,
            Issue::LongPath { repo, .. } => repo,
        }
    }
}

struct OwnerSummary {
    owner: String,
    total_repos: usize,
    issues: Vec<Issue>,
}

impl OwnerSummary {
    fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    fn issue_kinds(&self) -> HashSet<IssueKind> {
        self.issues.iter().map(|i| i.kind()).collect()
    }

    fn has_issue_kind(&self, kind: IssueKind) -> bool {
        self.issues.iter().any(|i| i.kind() == kind)
    }

    fn count_of_kind(&self, kind: IssueKind) -> usize {
        self.issues.iter().filter(|i| i.kind() == kind).count()
    }

    fn affected_repos_for_kind(&self, kind: IssueKind) -> usize {
        self.issues
            .iter()
            .filter(|i| i.kind() == kind)
            .map(|i| i.repo())
            .collect::<HashSet<_>>()
            .len()
    }
}

/// Create a new table with standard formatting
fn create_table() -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table
}

fn print_nfd_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of affected files:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path"]);

    for issue in issues {
        if let Issue::Nfd { repo, file_path } = issue {
            table.add_row(row![cell!(b -> repo), cell!(file_path)]);
        }
    }

    table.printstd();
}

fn print_case_duplicate_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of case-duplicates:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "Conflicting Files"]);

    for issue in issues {
        if let Issue::CaseDuplicate { repo, files } = issue {
            table.add_row(row![cell!(b -> repo), cell!(files.join("\n"))]);
        }
    }

    table.printstd();
}

fn print_large_files_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of large files:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path", "Size"]);

    for issue in issues {
        if let Issue::LargeFile {
            repo,
            file_path,
            size_bytes,
        } = issue
        {
            let size_mb = *size_bytes as f64 / BYTES_PER_MB;
            table.add_row(row![
                cell!(b -> repo),
                cell!(file_path),
                cell!(r -> format!("{:.1} MB", size_mb))
            ]);
        }
    }

    table.printstd();
}

fn print_large_ignored_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of files to remove:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path", "Size"]);

    for issue in issues {
        if let Issue::LargeIgnoredFile {
            repo,
            file_path,
            size_bytes,
        } = issue
        {
            let size_mb = *size_bytes as f64 / BYTES_PER_MB;
            table.add_row(row![
                cell!(b -> repo),
                cell!(file_path),
                cell!(r -> format!("{:.1} MB", size_mb))
            ]);
        }
    }

    table.printstd();
}

fn print_long_paths_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of long paths:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path", "Filename", "Path"]);

    for issue in issues {
        if let Issue::LongPath {
            repo,
            file_path,
            filename_bytes,
            path_bytes,
        } = issue
        {
            table.add_row(row![
                cell!(b -> repo),
                cell!(file_path),
                cell!(r -> format!("{}B", filename_bytes)),
                cell!(r -> format!("{}B", path_bytes))
            ]);
        }
    }

    table.printstd();
}

impl HealthCheckArgs {
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
                self.print_owner_summary(summary, false);
            }
            self.print_final_summary(&owner_summaries);
        } else {
            // Single owner: print details with recommendations
            if let Some(summary) = owner_summaries.first() {
                self.print_owner_summary(summary, true);
            }
        }

        // Run system configuration health checks
        self.print_system_health_checks();

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
        let results: Vec<Vec<Issue>> = common::process_with_progress(
            &progress_message,
            &repos,
            |repo_path| {
                check_repo(
                    repo_path,
                    threshold_bytes,
                    filename_threshold,
                    path_threshold,
                )
            },
            |_issues| String::new(), // Progress display doesn't need repo name from result
        );

        // Flatten all issues from all repos
        let issues: Vec<Issue> = results.into_iter().flatten().collect();

        Ok(OwnerSummary {
            owner: owner.to_string(),
            total_repos,
            issues,
        })
    }

    fn print_owner_summary(&self, summary: &OwnerSummary, include_recommendations: bool) {
        println!("\n{}", "═".repeat(LINE_WIDTH));
        println!("{} {}", "Owner:".bold(), summary.owner.cyan().bold());
        println!("{}", "═".repeat(LINE_WIDTH));

        if summary.is_clean() {
            println!(
                "{} All checks passed for {} repositories!",
                "✓".green().bold(),
                summary.total_repos
            );
        } else {
            // NFD issues section
            if summary.has_issue_kind(IssueKind::Nfd) {
                let count = summary.count_of_kind(IssueKind::Nfd);
                let repo_count = summary.affected_repos_for_kind(IssueKind::Nfd);

                println!(
                    "{} Found {} filenames with NFD normalization in {} of {} repositories",
                    "⚠".yellow().bold(),
                    count,
                    repo_count,
                    summary.total_repos
                );
                print_nfd_table(&summary.issues);
            }

            // Case duplicate section
            if summary.has_issue_kind(IssueKind::CaseDuplicate) {
                let count = summary.count_of_kind(IssueKind::CaseDuplicate);

                println!(
                    "\n{} Found {} case-duplicate file groups",
                    "⚠".yellow().bold(),
                    count
                );
                println!(
                    "{}",
                    "These will cause problems on case-insensitive filesystems (macOS/Windows)"
                        .dimmed()
                );
                print_case_duplicate_table(&summary.issues);
            }

            // Large files section
            if summary.has_issue_kind(IssueKind::LargeFile) {
                let count = summary.count_of_kind(IssueKind::LargeFile);
                let repo_count = summary.affected_repos_for_kind(IssueKind::LargeFile);

                println!(
                    "\n{} Found {} large files (> {} MB) not tracked by LFS in {} of {} repositories",
                    "⚠".yellow().bold(),
                    count,
                    self.large_file_mb,
                    repo_count,
                    summary.total_repos
                );
                print_large_files_table(&summary.issues);
            }

            // Large ignored files section (more serious - should be removed from git)
            if summary.has_issue_kind(IssueKind::LargeIgnoredFile) {
                let count = summary.count_of_kind(IssueKind::LargeIgnoredFile);
                let repo_count = summary.affected_repos_for_kind(IssueKind::LargeIgnoredFile);

                println!(
                    "\n{} Found {} large files (> {} MB) that should be removed from git in {} of {} repositories",
                    "⚠".red().bold(),
                    count,
                    self.large_file_mb,
                    repo_count,
                    summary.total_repos
                );
                println!(
                    "{}",
                    "These files match .gitignore patterns and should never have been committed"
                        .dimmed()
                );
                print_large_ignored_table(&summary.issues);
            }

            // Long paths section
            if summary.has_issue_kind(IssueKind::LongPath) {
                let count = summary.count_of_kind(IssueKind::LongPath);
                let repo_count = summary.affected_repos_for_kind(IssueKind::LongPath);

                println!(
                    "\n{} Found {} files with long paths or filenames (filename > {}B or path > {}B) in {} of {} repositories",
                    "⚠".yellow().bold(),
                    count,
                    self.filename_length_bytes,
                    self.path_length_bytes,
                    repo_count,
                    summary.total_repos
                );
                println!(
                    "{}",
                    "Long paths can cause checkout problems, especially on Windows".dimmed()
                );
                print_long_paths_table(&summary.issues);
            }

            if include_recommendations {
                self.print_recommendations(summary);
            }
        }
        println!("{}", "═".repeat(LINE_WIDTH));
    }

    fn print_final_summary(&self, summaries: &[OwnerSummary]) {
        println!("\n{}", "═".repeat(LINE_WIDTH));
        println!("{}", "FINAL SUMMARY".bold());
        println!("{}", "═".repeat(LINE_WIDTH));

        let total_repos: usize = summaries.iter().map(|s| s.total_repos).sum();

        // Count issues by kind across all summaries
        let total_nfd: usize = summaries
            .iter()
            .map(|s| s.count_of_kind(IssueKind::Nfd))
            .sum();
        let total_case_dups: usize = summaries
            .iter()
            .map(|s| s.count_of_kind(IssueKind::CaseDuplicate))
            .sum();
        let total_large_files: usize = summaries
            .iter()
            .map(|s| s.count_of_kind(IssueKind::LargeFile))
            .sum();
        let total_large_ignored: usize = summaries
            .iter()
            .map(|s| s.count_of_kind(IssueKind::LargeIgnoredFile))
            .sum();
        let total_long_paths: usize = summaries
            .iter()
            .map(|s| s.count_of_kind(IssueKind::LongPath))
            .sum();

        let all_clean = total_nfd == 0
            && total_case_dups == 0
            && total_large_files == 0
            && total_large_ignored == 0
            && total_long_paths == 0;

        if all_clean {
            println!(
                "{} All checks passed for {} repositories across {} owners!",
                "✓".green().bold(),
                total_repos,
                summaries.len()
            );
        } else {
            if total_nfd > 0 {
                println!(
                    "{} Found {} filenames with NFD normalization across {} owners",
                    "⚠".yellow().bold(),
                    total_nfd,
                    summaries.len()
                );
            }

            if total_case_dups > 0 {
                println!(
                    "{} Found {} case-duplicate file groups across {} owners",
                    "⚠".yellow().bold(),
                    total_case_dups,
                    summaries.len()
                );
            }

            if total_large_files > 0 {
                println!(
                    "{} Found {} large files (> {} MB) not tracked by LFS across {} owners",
                    "⚠".yellow().bold(),
                    total_large_files,
                    self.large_file_mb,
                    summaries.len()
                );
            }

            if total_large_ignored > 0 {
                println!(
                    "{} Found {} large files (> {} MB) that should be removed from git across {} owners",
                    "⚠".red().bold(),
                    total_large_ignored,
                    self.large_file_mb,
                    summaries.len()
                );
            }

            if total_long_paths > 0 {
                println!(
                    "{} Found {} files with long paths or filenames (>{}B filename or >{}B path) across {} owners",
                    "⚠".yellow().bold(),
                    total_long_paths,
                    self.filename_length_bytes,
                    self.path_length_bytes,
                    summaries.len()
                );
            }

            // Create a combined summary for recommendations
            let combined_issues: Vec<Issue> = summaries
                .iter()
                .flat_map(|s| s.issues.iter().cloned())
                .collect();
            let combined_summary = OwnerSummary {
                owner: String::new(),
                total_repos,
                issues: combined_issues,
            };
            self.print_recommendations(&combined_summary);
        }
        println!("{}", "═".repeat(LINE_WIDTH));
    }

    fn print_recommendations(&self, summary: &OwnerSummary) {
        if summary.is_clean() {
            return;
        }

        println!("\n{}", "Recommendations:".bold());

        let kinds = summary.issue_kinds();

        for kind in &kinds {
            match kind {
                IssueKind::Nfd => {
                    println!("\n{}", "For NFD normalization issues:".yellow());
                    println!(
                        "  1. Ensure {} is set on macOS:",
                        "git config --global core.precomposeUnicode true".cyan()
                    );
                    println!(
                        "     {}",
                        "git config --global core.precomposeUnicode true".cyan()
                    );
                    println!(
                        "  2. Use {} to fix affected repositories:",
                        "nfd-fixer".cyan()
                    );
                    println!("     {}", "https://github.com/divvun/nfd-fixer".cyan());
                }
                IssueKind::CaseDuplicate => {
                    println!("\n{}", "For case-duplicate issues:".yellow());
                    println!(
                        "  1. These files have the same name with different case (e.g., File.txt and file.txt)"
                    );
                    println!(
                        "  2. On case-insensitive filesystems (macOS/Windows), only one can exist"
                    );
                    println!("  3. Git-LFS gets confused and may check out the wrong version");
                    println!("  4. To fix: On a case-sensitive Linux system:");
                    println!("     - Identify which variant to keep");
                    println!(
                        "     - Delete the unwanted variant(s): {}",
                        "git rm <unwanted_file>".cyan()
                    );
                    println!("     - Commit and push the change");
                }
                IssueKind::LargeFile => {
                    println!("\n{}", "For large files not tracked by LFS:".yellow());
                    println!("  1. Install Git LFS if not already installed:");
                    println!("     {}", "brew install git-lfs && git lfs install".cyan());
                    println!("  2. Navigate to the repository and track the file type:");
                    println!("     {}", "git lfs track \"*.extension\"".cyan());
                    println!(
                        "     (Replace .extension with the actual file extension, e.g., .zip, .pdf, .bin)"
                    );
                    println!("  3. Or track a specific file:");
                    println!("     {}", "git lfs track \"path/to/large/file.ext\"".cyan());
                    println!("  4. Add the .gitattributes file:");
                    println!("     {}", "git add .gitattributes".cyan());
                    println!("  5. Remove the file from Git's object database and re-add it:");
                    println!("     {}", "git rm --cached path/to/large/file.ext".cyan());
                    println!("     {}", "git add path/to/large/file.ext".cyan());
                    println!("  6. Commit and push:");
                    println!("     {}", "git commit -m \"Move large file to LFS\"".cyan());
                    println!("     {}", "git push".cyan());
                    println!("  7. To clean up old large files from history, use:");
                    println!(
                        "     {}",
                        "git lfs migrate import --include=\"*.extension\" --everything".cyan()
                    );
                    println!(
                        "     {}",
                        "Note: This rewrites history. Coordinate with team before running."
                            .dimmed()
                    );
                }
                IssueKind::LargeIgnoredFile => {
                    println!(
                        "\n{}",
                        "For large files that should be removed from git:".red()
                    );
                    println!(
                        "  {} These files match .gitignore patterns and should never have been committed",
                        "!".red().bold()
                    );
                    println!("  1. Remove the file from git (but keep it locally):");
                    println!("     {}", "git rm --cached path/to/file".cyan());
                    println!("  2. Verify the file is now in .gitignore:");
                    println!("     {}", "git check-ignore path/to/file".cyan());
                    println!("     (Should output the file path if properly ignored)");
                    println!("  3. Commit the removal:");
                    println!(
                        "     {}",
                        "git commit -m \"Remove generated file from git\"".cyan()
                    );
                    println!("     {}", "git push".cyan());
                    println!("  4. To completely remove from history (reduces repo size):");
                    println!(
                        "     {}",
                        "git filter-repo --path path/to/file --invert-paths".cyan()
                    );
                    println!(
                        "     {} or use BFG Repo-Cleaner for multiple files",
                        "OR".bold()
                    );
                    println!(
                        "     {}",
                        "Note: This rewrites history. All team members must re-clone."
                            .red()
                            .dimmed()
                    );
                }
                IssueKind::LongPath => {
                    println!("\n{}", "For files with long paths or filenames:".yellow());
                    println!(
                        "  {} Long paths can cause checkout problems, especially on Windows (260 char limit)",
                        "⚠".yellow().bold()
                    );
                    println!(
                        "  {} NFD normalization makes paths appear shorter than they are in bytes",
                        "Note:".dimmed()
                    );
                    println!();
                    println!("  {}", "Strategies to shorten paths:".bold());
                    println!("  1. Shorten directory names in deep hierarchies:");
                    println!(
                        "     {}",
                        "tools/grammarcheckers/errordata/realworderrors/".dimmed()
                    );
                    println!("     {} {}", "→".cyan(), "tools/gc/errors/realword/".cyan());
                    println!("  2. Flatten directory structures where possible:");
                    println!(
                        "     {}",
                        "src/components/common/utils/helpers/string/".dimmed()
                    );
                    println!("     {} {}", "→".cyan(), "src/utils/string/".cyan());
                    println!("  3. Shorten filenames, especially for files deep in the tree:");
                    println!(
                        "     {}",
                        "very_long_descriptive_filename_with_many_words.txt".dimmed()
                    );
                    println!("     {} {}", "→".cyan(), "descriptive_file.txt".cyan());
                    println!("  4. Use abbreviations consistently:");
                    println!("     {}", "documentation/ → docs/".cyan());
                    println!("     {}", "configuration/ → config/".cyan());
                    println!("     {}", "resources/ → res/".cyan());
                    println!();
                    println!("  {}", "To rename and preserve history:".bold());
                    println!(
                        "     {}",
                        "git mv old/very/long/path/file.txt shorter/path/file.txt".cyan()
                    );
                    println!(
                        "     {}",
                        "git commit -m \"Shorten path for compatibility\"".cyan()
                    );
                }
            }
        }
    }

    fn print_system_health_checks(&self) {
        println!("\n{}", "═".repeat(LINE_WIDTH));
        println!("{}", "SYSTEM CONFIGURATION CHECKS".bold());
        println!("{}", "═".repeat(LINE_WIDTH));

        let warnings = health::check_git_config();

        // Print status for each check
        println!("\n{}", "System configuration status:".bold());

        // Check 1: Git version
        let has_git_version_issue = warnings.iter().any(|w| w.title.contains("Git version"));

        let git_version = health::get_git_version().unwrap_or_else(|| "unknown".to_string());

        if has_git_version_issue {
            println!(
                "  {} {} ({})",
                "✗".red().bold(),
                "Git version".dimmed(),
                git_version.dimmed()
            );
        } else {
            println!(
                "  {} {} ({})",
                "✓".green().bold(),
                "Git version",
                git_version.bright_black()
            );
        }

        // Check 2: core.precomposeUnicode (macOS only)
        if cfg!(target_os = "macos") {
            let has_precompose_issue = warnings
                .iter()
                .any(|w| w.title.contains("precomposeUnicode"));

            let precompose_value = health::get_precompose_unicode_value();

            if has_precompose_issue {
                println!(
                    "  {} {} ({})",
                    "✗".red().bold(),
                    "core.precomposeUnicode setting".dimmed(),
                    precompose_value.dimmed()
                );
            } else {
                println!(
                    "  {} {} ({})",
                    "✓".green().bold(),
                    "core.precomposeUnicode setting",
                    precompose_value.bright_black()
                );
            }
        }

        // Check 3: core.autocrlf (Unix systems only)
        if cfg!(unix) {
            let has_autocrlf_issue = warnings.iter().any(|w| w.title.contains("autocrlf"));

            let autocrlf_value = health::get_autocrlf_value();

            if has_autocrlf_issue {
                println!(
                    "  {} {} ({})",
                    "✗".red().bold(),
                    "core.autocrlf setting".dimmed(),
                    autocrlf_value.dimmed()
                );
            } else {
                println!(
                    "  {} {} ({})",
                    "✓".green().bold(),
                    "core.autocrlf setting",
                    autocrlf_value.bright_black()
                );
            }
        }

        // Check 4: Git LFS installation
        let has_lfs_issue = warnings.iter().any(|w| w.title.contains("Git LFS"));

        if has_lfs_issue {
            println!(
                "  {} {} ({})",
                "✗".red().bold(),
                "Git LFS installation".dimmed(),
                "not installed".dimmed()
            );
        } else {
            println!(
                "  {} {} ({})",
                "✓".green().bold(),
                "Git LFS installation",
                "installed".bright_black()
            );
        }

        // Print remediation steps if there are issues
        if !warnings.is_empty() {
            println!("\n{}", "Configuration issues found:".yellow().bold());

            for warning in &warnings {
                println!("\n  {} {}", "⚠".yellow().bold(), warning.title.yellow());
                println!("    {}", warning.message);
                if let Some(suggestion) = &warning.suggestion {
                    println!("\n    {}", "How to fix:".bold());
                    for line in suggestion.lines() {
                        println!("    {}", line.bright_black());
                    }
                }
            }
        }

        println!("\n{}", "═".repeat(LINE_WIDTH));
    }
}

/// Check a single repository for all issue types
fn check_repo(
    repo_path: &PathBuf,
    large_file_threshold: u64,
    filename_threshold: usize,
    path_threshold: usize,
) -> Vec<Issue> {
    let repo_name = path::dir_name(repo_path).unwrap_or_default();

    // Try to open as git repo
    let git_repo = match git::open(repo_path) {
        Ok(r) => r,
        Err(e) => {
            log::debug!("Skipping {}: not a git repository ({})", repo_name, e);
            return Vec::new();
        }
    };

    let mut issues = Vec::new();

    match check_repo_for_nfc_issues(&git_repo, &repo_name) {
        Ok(nfc_issues) => issues.extend(nfc_issues),
        Err(e) => log::debug!("NFC check failed for {}: {}", repo_name, e),
    }

    match check_repo_for_case_duplicates(&git_repo, &repo_name) {
        Ok(case_issues) => issues.extend(case_issues),
        Err(e) => log::debug!("Case duplicate check failed for {}: {}", repo_name, e),
    }

    match check_repo_for_large_files_and_long_paths(
        &git_repo,
        &repo_name,
        large_file_threshold,
        filename_threshold,
        path_threshold,
    ) {
        Ok(large_path_issues) => issues.extend(large_path_issues),
        Err(e) => log::debug!(
            "Large files/long paths check failed for {}: {}",
            repo_name,
            e
        ),
    }

    issues
}

/// Build a full file path from tree walk path prefix and entry name
fn build_full_path(path_prefix: &str, name: &str) -> String {
    if path_prefix.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", path_prefix.trim_end_matches('/'), name)
    }
}

/// Get the HEAD tree from a repository, returning None for empty repos
fn get_head_tree(repo: &git2::Repository) -> Option<git2::Tree<'_>> {
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    commit.tree().ok()
}

/// Check a single repository for NFC normalization issues
///
/// This function walks the git tree and identifies filenames that are stored in NFD
/// (decomposed) form when an NFC (composed) equivalent exists in Unicode.
///
/// Note: Some character combinations (like Cyrillic base characters + combining macron U+0304)
/// have NO precomposed NFC form in Unicode. These are correctly stored in NFD form and will
/// NOT be flagged as issues. The function only reports files where an NFC equivalent exists
/// but the filename uses NFD instead.
fn check_repo_for_nfc_issues(git_repo: &git2::Repository, repo_name: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    let tree = match get_head_tree(git_repo) {
        Some(t) => t,
        None => return Ok(issues), // Empty repo or no commits
    };

    let repo_name = repo_name.to_string();

    // Walk the tree recursively
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            // Use name_bytes() to get raw bytes from git object database without normalization.
            // The name() method might apply NFC normalization depending on git config.
            let name_bytes = entry.name_bytes();

            // Check if name_bytes is valid UTF-8 and compare with NFC form
            if let Ok(name_str) = std::str::from_utf8(name_bytes) {
                let normalized: String = name_str.nfc().collect();

                // Only flag as issue if NFC form differs from current form.
                // This means an NFC equivalent exists but the file uses NFD.
                if name_str != normalized.as_str() {
                    issues.push(Issue::Nfd {
                        repo: repo_name.clone(),
                        file_path: build_full_path(path, name_str),
                    });
                }
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(issues)
}

/// Check a single repository for case-duplicate files
///
/// This function walks the git tree and identifies files that have the same name
/// when compared case-insensitively. On case-sensitive filesystems (Linux), these
/// files can coexist, but on case-insensitive filesystems (macOS/Windows), only
/// one can exist at a time. This causes confusion for git-lfs, which may check out
/// the wrong version.
///
/// Example: "File.txt" and "file.txt" are different on Linux but the same on macOS.
fn check_repo_for_case_duplicates(
    git_repo: &git2::Repository,
    repo_name: &str,
) -> Result<Vec<Issue>> {
    // Map lowercase path -> list of actual paths
    let mut path_map: HashMap<String, Vec<String>> = HashMap::new();

    let tree = match get_head_tree(git_repo) {
        Some(t) => t,
        None => return Ok(Vec::new()), // Empty repo or no commits
    };

    // Walk the tree and collect all file paths
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            if let Ok(name_str) = std::str::from_utf8(entry.name_bytes()) {
                let full_path = build_full_path(path, name_str);

                // Use lowercase version as key for case-insensitive comparison
                let lowercase_path = full_path.to_lowercase();
                path_map.entry(lowercase_path).or_default().push(full_path);
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    // Find entries with more than one variant and convert to Issues
    let mut issues = Vec::new();
    let mut duplicates: Vec<Vec<String>> = path_map
        .into_values()
        .filter(|paths| paths.len() > 1)
        .collect();

    // Sort for consistent output
    duplicates.sort();

    for files in duplicates {
        issues.push(Issue::CaseDuplicate {
            repo: repo_name.to_string(),
            files,
        });
    }

    Ok(issues)
}

/// Check a single repository for large files not tracked by LFS and long paths
///
/// This function walks the git tree and identifies:
/// 1. Large files that should be tracked by LFS
/// 2. Large files that match .gitignore patterns (should be removed from git entirely)
/// 3. Files with long paths or filenames
fn check_repo_for_large_files_and_long_paths(
    git_repo: &git2::Repository,
    repo_name: &str,
    threshold_bytes: u64,
    filename_threshold: usize,
    path_threshold: usize,
) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    let tree = match get_head_tree(git_repo) {
        Some(t) => t,
        None => return Ok(issues), // Empty repo or no commits
    };

    let repo_name = repo_name.to_string();

    // Collect issues during tree walk, then sort after
    let mut large_files: Vec<(String, u64)> = Vec::new();
    let mut large_ignored_files: Vec<(String, u64)> = Vec::new();
    let mut long_paths: Vec<(String, usize, usize)> = Vec::new();

    // Walk the tree recursively
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            let name = std::str::from_utf8(entry.name_bytes()).unwrap_or("<invalid utf-8>");
            let full_path = build_full_path(path, name);

            // Check path and filename lengths
            let path_bytes_len = full_path.as_bytes().len();
            let filename_bytes_len = name.as_bytes().len();

            if filename_bytes_len > filename_threshold || path_bytes_len > path_threshold {
                long_paths.push((full_path.clone(), path_bytes_len, filename_bytes_len));
            }

            // Get the blob object to check its size
            let oid: git2::Oid = entry.id().into();
            if let Ok(blob) = git_repo.find_blob(oid) {
                let size = blob.size();

                // Check if file exceeds threshold
                if size > threshold_bytes as usize {
                    // Check if it's an LFS pointer file
                    // LFS pointer files are small text files with specific format
                    let is_lfs = size < LFS_POINTER_MAX_BYTES
                        && blob.content().starts_with(LFS_POINTER_PREFIX);

                    if !is_lfs {
                        // Check if file should be ignored according to .gitignore
                        let should_ignore = git_repo
                            .status_should_ignore(std::path::Path::new(&full_path))
                            .unwrap_or(false);

                        if should_ignore {
                            large_ignored_files.push((full_path, size as u64));
                        } else {
                            large_files.push((full_path, size as u64));
                        }
                    }
                }
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    // Sort by size (largest first) and convert to Issues
    large_files.sort_by(|a, b| b.1.cmp(&a.1));
    for (file_path, size_bytes) in large_files {
        issues.push(Issue::LargeFile {
            repo: repo_name.clone(),
            file_path,
            size_bytes,
        });
    }

    large_ignored_files.sort_by(|a, b| b.1.cmp(&a.1));
    for (file_path, size_bytes) in large_ignored_files {
        issues.push(Issue::LargeIgnoredFile {
            repo: repo_name.clone(),
            file_path,
            size_bytes,
        });
    }

    // Sort long paths by path length (longest first)
    long_paths.sort_by(|a, b| b.1.cmp(&a.1));
    for (file_path, path_bytes, filename_bytes) in long_paths {
        issues.push(Issue::LongPath {
            repo: repo_name.clone(),
            file_path,
            path_bytes,
            filename_bytes,
        });
    }

    Ok(issues)
}
