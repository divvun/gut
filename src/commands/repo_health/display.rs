use super::RepoHealthArgs;
use super::types::{BYTES_PER_MB, Issue, IssueData, IssueKind, LINE_WIDTH, OwnerSummary};
use crate::system_health;
use colored::*;
use prettytable::{Table, cell, format, row};

/// Create a new table with standard formatting
fn create_table() -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table
}

pub fn print_nfd_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of affected files:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path"]);

    for issue in issues {
        if let IssueData::Nfd { file_path } = &issue.data {
            table.add_row(row![cell!(b -> &issue.repo), cell!(file_path)]);
        }
    }

    table.printstd();
}

pub fn print_case_duplicate_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of case-duplicates:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "Conflicting Files"]);

    for issue in issues {
        if let IssueData::CaseDuplicate { files } = &issue.data {
            table.add_row(row![cell!(b -> &issue.repo), cell!(files.join("\n"))]);
        }
    }

    table.printstd();
}

pub fn print_large_files_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of large files:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path", "Size"]);

    for issue in issues {
        if let IssueData::LargeFile {
            file_path,
            size_bytes,
        } = &issue.data
        {
            let size_mb = *size_bytes as f64 / BYTES_PER_MB;
            table.add_row(row![
                cell!(b -> &issue.repo),
                cell!(file_path),
                cell!(r -> format!("{:.1}MB", size_mb))
            ]);
        }
    }

    table.printstd();
}

pub fn print_large_ignored_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of files to remove:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path", "Size"]);

    for issue in issues {
        if let IssueData::LargeIgnoredFile {
            file_path,
            size_bytes,
        } = &issue.data
        {
            let size_mb = *size_bytes as f64 / BYTES_PER_MB;
            table.add_row(row![
                cell!(b -> &issue.repo),
                cell!(file_path),
                cell!(r -> format!("{:.1}MB", size_mb))
            ]);
        }
    }

    table.printstd();
}

pub fn print_long_paths_table(issues: &[Issue]) {
    println!("\n{}", "Detailed list of long paths:".bold());
    let mut table = create_table();
    table.set_titles(row!["Repository", "File Path", "Filename", "Path"]);

    for issue in issues {
        if let IssueData::LongPath {
            file_path,
            path_bytes,
            filename_bytes,
        } = &issue.data
        {
            table.add_row(row![
                cell!(b -> &issue.repo),
                cell!(file_path),
                cell!(r -> format!("{}B", filename_bytes)),
                cell!(r -> format!("{}B", path_bytes))
            ]);
        }
    }

    table.printstd();
}

pub fn print_owner_summary(
    args: &RepoHealthArgs,
    summary: &OwnerSummary,
    include_recommendations: bool,
) {
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
                "\n{} Found {} large files (> {}MB) not tracked by LFS in {} of {} repositories",
                "⚠".yellow().bold(),
                count,
                args.large_file_mb,
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
                "\n{} Found {} large files (> {}MB) that should be removed from git in {} of {} repositories",
                "⚠".red().bold(),
                count,
                args.large_file_mb,
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
                args.filename_length_bytes,
                args.path_length_bytes,
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
            print_recommendations(args, summary);
        }
    }
    println!("{}", "═".repeat(LINE_WIDTH));
}

pub fn print_final_summary(args: &RepoHealthArgs, summaries: &[OwnerSummary]) {
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
                "{} Found {} large files (> {}MB) not tracked by LFS across {} owners",
                "⚠".yellow().bold(),
                total_large_files,
                args.large_file_mb,
                summaries.len()
            );
        }

        if total_large_ignored > 0 {
            println!(
                "{} Found {} large files (> {}MB) that should be removed from git across {} owners",
                "⚠".red().bold(),
                total_large_ignored,
                args.large_file_mb,
                summaries.len()
            );
        }

        if total_long_paths > 0 {
            println!(
                "{} Found {} files with long paths or filenames (>{}B filename or >{}B path) across {} owners",
                "⚠".yellow().bold(),
                total_long_paths,
                args.filename_length_bytes,
                args.path_length_bytes,
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
        print_recommendations(args, &combined_summary);
    }
    println!("{}", "═".repeat(LINE_WIDTH));
}

pub fn print_recommendations(_args: &RepoHealthArgs, summary: &OwnerSummary) {
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
                    "Note: This rewrites history. Coordinate with team before running.".dimmed()
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

pub fn print_system_health_checks() {
    println!("\n{}", "═".repeat(LINE_WIDTH));
    println!("{}", "SYSTEM CONFIGURATION CHECKS".bold());
    println!("{}", "═".repeat(LINE_WIDTH));

    let warnings = system_health::check_git_config();

    // Print status for each check
    println!("\n{}", "System configuration status:".bold());

    // Check 1: Git version
    let has_git_version_issue = warnings.iter().any(|w| w.title.contains("Git version"));

    let git_version = system_health::get_git_version().unwrap_or_else(|| "unknown".to_string());

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

        let precompose_value = system_health::get_precompose_unicode_value();

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

        let autocrlf_value = system_health::get_autocrlf_value();

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
