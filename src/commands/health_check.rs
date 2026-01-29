use super::common;
use crate::git;
use crate::health;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use prettytable::{Table, cell, format, row};
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Parser)]
/// Check repositories for NFD normalization issues and case-duplicate filenames
///
/// This command scans all files in local repositories and checks for:
///
/// - NFD normalization issues: Filenames with decomposed Unicode characters that
///   have a composed (NFC) equivalent, which can cause conflicts on macOS
///
/// - Case-duplicate filenames: Files with identical names except for letter case
///   (e.g., File.txt and file.txt), which cause problems on case-insensitive
///   filesystems like macOS and Windows
pub struct HealthCheckArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

#[derive(Debug)]
struct NormalizationIssue {
    owner: String,
    repo: String,
    file_path: String,
}

#[derive(Debug)]
struct CaseDuplicateIssue {
    owner: String,
    repo: String,
    files: Vec<String>,
}

struct RepoCheckResult {
    repo_name: String,
    nfd_issues: Vec<String>,
    case_duplicates: Vec<Vec<String>>,
}

struct OwnerSummary {
    owner: String,
    total_repos: usize,
    nfd_issues: Vec<NormalizationIssue>,
    case_duplicates: Vec<CaseDuplicateIssue>,
}

impl HealthCheckArgs {
    pub fn run(&self) -> Result<()> {
        let _user = common::user()?;
        let root = common::root()?;

        let owners = if self.all_owners {
            common::get_all_owners()?
        } else {
            vec![common::owner(self.owner.as_deref())?]
        };

        let mut owner_summaries = Vec::new();

        for (index, owner) in owners.iter().enumerate() {
            // Add blank line before each owner (except first) when checking multiple
            if self.all_owners && index > 0 {
                println!();
            }
            
            let summary = self.check_owner(&root, owner)?;
            
            // Print per-owner summary if checking multiple owners
            if self.all_owners {
                self.print_owner_summary(&summary);
            }
            
            owner_summaries.push(summary);
        }

        // Print final summary
        if self.all_owners {
            self.print_final_summary(&owner_summaries);
        } else {
            // Single owner - just print the summary
            if let Some(summary) = owner_summaries.first() {
                self.print_single_owner_summary(summary);
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
                nfd_issues: Vec::new(),
                case_duplicates: Vec::new(),
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
        let results = common::process_with_progress(
            &progress_message,
            &repos,
            |repo_path| check_repo(repo_path),
            |result| result.repo_name.clone(),
        );

        // Collect all issues
        let mut all_nfd_issues = Vec::new();
        let mut all_case_duplicates = Vec::new();
        
        for result in results {
            for file_path in result.nfd_issues {
                all_nfd_issues.push(NormalizationIssue {
                    owner: owner.to_string(),
                    repo: result.repo_name.clone(),
                    file_path,
                });
            }
            
            for duplicate_group in result.case_duplicates {
                all_case_duplicates.push(CaseDuplicateIssue {
                    owner: owner.to_string(),
                    repo: result.repo_name.clone(),
                    files: duplicate_group,
                });
            }
        }

        Ok(OwnerSummary {
            owner: owner.to_string(),
            total_repos,
            nfd_issues: all_nfd_issues,
            case_duplicates: all_case_duplicates,
        })
    }

    fn print_owner_summary(&self, summary: &OwnerSummary) {
        println!("{} {}:", "Owner:".bold(), summary.owner.cyan().bold());
        
        if summary.nfd_issues.is_empty() && summary.case_duplicates.is_empty() {
            println!("  {} All filenames are correctly encoded", "✓".green().bold());
        } else {
            // Report NFD issues
            if !summary.nfd_issues.is_empty() {
                let repo_count = summary.nfd_issues.iter()
                    .map(|i| i.repo.as_str())
                    .collect::<std::collections::HashSet<_>>()
                    .len();
                
                println!("  {} Found {} filenames with NFD normalization in {} repositories", 
                    "⚠".yellow().bold(),
                    summary.nfd_issues.len(),
                    repo_count
                );
                
                // Group by repo
                let mut by_repo: std::collections::HashMap<String, Vec<&NormalizationIssue>> = 
                    std::collections::HashMap::new();
                
                for issue in &summary.nfd_issues {
                    by_repo.entry(issue.repo.clone()).or_default().push(issue);
                }
                
                let mut repos: Vec<_> = by_repo.keys().collect();
                repos.sort();
                
                for repo in repos {
                    let issues = &by_repo[repo];
                    println!("    {} {} ({} files)", "→".cyan(), repo.yellow(), issues.len());
                    for issue in issues {
                        println!("      {}", issue.file_path.dimmed());
                    }
                }
            }
            
            // Report case duplicates
            if !summary.case_duplicates.is_empty() {
                println!("\n  {} Found {} case-duplicate file groups (problematic on macOS/Windows)", 
                    "⚠".yellow().bold(),
                    summary.case_duplicates.len()
                );
                
                for dup in &summary.case_duplicates {
                    println!("    {} {} ({} variants)", "→".cyan(), dup.repo.yellow(), dup.files.len());
                    for file in &dup.files {
                        println!("      {}", file.dimmed());
                    }
                }
            }
        }
    }

    fn print_single_owner_summary(&self, summary: &OwnerSummary) {
        println!("\n{}", "═".repeat(80));
        println!("{} {}", "Owner:".bold(), summary.owner.cyan().bold());
        println!("{}", "═".repeat(80));
        
        if summary.nfd_issues.is_empty() && summary.case_duplicates.is_empty() {
            println!("{} All filenames are correctly encoded in {} repositories!", 
                "✓".green().bold(),
                summary.total_repos
            );
        } else {
            // NFD issues section
            if !summary.nfd_issues.is_empty() {
                let repo_count = summary.nfd_issues.iter()
                    .map(|i| i.repo.as_str())
                    .collect::<std::collections::HashSet<_>>()
                    .len();
                
                println!("{} Found {} filenames with NFD normalization in {} of {} repositories", 
                    "⚠".yellow().bold(),
                    summary.nfd_issues.len(),
                    repo_count,
                    summary.total_repos
                );
                
                // Print detailed table
                println!("\n{}", "Detailed list of affected files:".bold());
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
                table.set_titles(row!["Repository", "File Path"]);
                
                for issue in &summary.nfd_issues {
                    table.add_row(row![
                        cell!(b -> &issue.repo),
                        cell!(&issue.file_path)
                    ]);
                }
                
                table.printstd();
            }
            
            // Case duplicate section
            if !summary.case_duplicates.is_empty() {
                println!("\n{} Found {} case-duplicate file groups", 
                    "⚠".yellow().bold(),
                    summary.case_duplicates.len()
                );
                println!("{}", "These will cause problems on case-insensitive filesystems (macOS/Windows)".dimmed());
                
                println!("\n{}", "Detailed list of case-duplicates:".bold());
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
                table.set_titles(row!["Repository", "Conflicting Files"]);
                
                for dup in &summary.case_duplicates {
                    table.add_row(row![
                        cell!(b -> &dup.repo),
                        cell!(dup.files.join("\n"))
                    ]);
                }
                
                table.printstd();
            }
            
            self.print_recommendations(!summary.nfd_issues.is_empty(), !summary.case_duplicates.is_empty());
        }
        println!("{}", "═".repeat(80));
    }

    fn print_final_summary(&self, summaries: &[OwnerSummary]) {
        println!("\n{}", "═".repeat(80));
        println!("{}", "FINAL SUMMARY".bold());
        println!("{}", "═".repeat(80));
        
        let total_repos: usize = summaries.iter().map(|s| s.total_repos).sum();
        let total_nfd: usize = summaries.iter().map(|s| s.nfd_issues.len()).sum();
        let total_case_dups: usize = summaries.iter().map(|s| s.case_duplicates.len()).sum();
        
        if total_nfd == 0 && total_case_dups == 0 {
            println!("{} All filenames are correctly encoded in {} repositories across {} owners!", 
                "✓".green().bold(),
                total_repos,
                summaries.len()
            );
        } else {
            if total_nfd > 0 {
                println!("{} Found {} filenames with NFD normalization across {} owners", 
                    "⚠".yellow().bold(),
                    total_nfd,
                    summaries.len()
                );
            }
            
            if total_case_dups > 0 {
                println!("{} Found {} case-duplicate file groups across {} owners", 
                    "⚠".yellow().bold(),
                    total_case_dups,
                    summaries.len()
                );
            }
            
            self.print_recommendations(total_nfd > 0, total_case_dups > 0);
        }
        println!("{}", "═".repeat(80));
    }

    fn print_recommendations(&self, has_nfd_issues: bool, has_case_duplicates: bool) {
        if !has_nfd_issues && !has_case_duplicates {
            return;
        }
        
        println!("\n{}", "Recommendations:".bold());
        
        if has_nfd_issues {
            println!("\n{}", "For NFD normalization issues:".yellow());
            println!("  1. Ensure {} is set on macOS:", "git config --global core.precomposeUnicode true".cyan());
            println!("     {}", "git config --global core.precomposeUnicode true".cyan());
            println!("  2. Use {} to fix affected repositories:", "nfd-fixer".cyan());
            println!("     {}", "https://github.com/divvun/nfd-fixer".cyan());
        }
        
        if has_case_duplicates {
            println!("\n{}", "For case-duplicate issues:".yellow());
            println!("  1. These files have the same name with different case (e.g., File.txt and file.txt)");
            println!("  2. On case-insensitive filesystems (macOS/Windows), only one can exist");
            println!("  3. Git-LFS gets confused and may check out the wrong version");
            println!("  4. To fix: On a case-sensitive Linux system:");
            println!("     - Identify which variant to keep");
            println!("     - Delete the unwanted variant(s): {}", "git rm <unwanted_file>".cyan());
            println!("     - Commit and push the change");
        }
    }

    fn print_system_health_checks(&self) {
        println!("\n{}", "═".repeat(80));
        println!("{}", "SYSTEM CONFIGURATION CHECKS".bold());
        println!("{}", "═".repeat(80));

        let warnings = health::check_git_config();
        
        // Print status for each check
        println!("\n{}", "System configuration status:".bold());
        
        // Check 1: Git version
        let has_git_version_issue = warnings.iter()
            .any(|w| w.title.contains("Git version"));
        
        let git_version = health::get_git_version()
            .unwrap_or_else(|| "unknown".to_string());
        
        if has_git_version_issue {
            println!("  {} {} ({})", 
                "✗".red().bold(), 
                "Git version".dimmed(),
                git_version.dimmed()
            );
        } else {
            println!("  {} {} ({})", 
                "✓".green().bold(), 
                "Git version",
                git_version.bright_black()
            );
        }
        
        // Check 2: core.precomposeUnicode (macOS only)
        if cfg!(target_os = "macos") {
            let has_precompose_issue = warnings.iter()
                .any(|w| w.title.contains("precomposeUnicode"));
            
            let precompose_value = health::get_precompose_unicode_value();
            
            if has_precompose_issue {
                println!("  {} {} ({})", 
                    "✗".red().bold(), 
                    "core.precomposeUnicode setting".dimmed(),
                    precompose_value.dimmed()
                );
            } else {
                println!("  {} {} ({})", 
                    "✓".green().bold(), 
                    "core.precomposeUnicode setting",
                    precompose_value.bright_black()
                );
            }
        }
        
        // Check 3: Git LFS installation
        let has_lfs_issue = warnings.iter()
            .any(|w| w.title.contains("Git LFS"));
        
        let lfs_installed = health::is_git_lfs_installed();
        
        if has_lfs_issue {
            println!("  {} {} ({})", 
                "✗".red().bold(), 
                "Git LFS installation".dimmed(),
                "not installed".dimmed()
            );
        } else {
            println!("  {} {} ({})", 
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
        
        println!("\n{}", "═".repeat(80));
    }
}

/// Check a single repository for NFC normalization issues
fn check_repo(repo_path: &PathBuf) -> RepoCheckResult {
    let repo_name = path::dir_name(repo_path).unwrap_or_default();
    
    // Try to open as git repo
    let git_repo = match git::open(repo_path) {
        Ok(r) => r,
        Err(_) => return RepoCheckResult {
            repo_name,
            nfd_issues: Vec::new(),
            case_duplicates: Vec::new(),
        },
    };

    let nfd_issues = check_repo_for_nfc_issues(&git_repo).unwrap_or_default();
    let case_duplicates = check_repo_for_case_duplicates(&git_repo).unwrap_or_default();
    
    RepoCheckResult {
        repo_name,
        nfd_issues,
        case_duplicates,
    }
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
fn check_repo_for_nfc_issues(git_repo: &git2::Repository) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    
    // Get the HEAD tree
    let head = match git_repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(issues), // Empty repo or no commits
    };
    
    let commit = head.peel_to_commit()?;
    let tree = commit.tree()?;
    
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
                    let full_path = if path.is_empty() {
                        name_str.to_string()
                    } else {
                        format!("{}/{}", path, name_str)
                    };
                    issues.push(full_path);
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
fn check_repo_for_case_duplicates(git_repo: &git2::Repository) -> Result<Vec<Vec<String>>> {
    use std::collections::HashMap;
    
    // Map lowercase path -> list of actual paths
    let mut path_map: HashMap<String, Vec<String>> = HashMap::new();
    
    // Get the HEAD tree
    let head = match git_repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(Vec::new()), // Empty repo or no commits
    };
    
    let commit = head.peel_to_commit()?;
    let tree = commit.tree()?;
    
    // Walk the tree and collect all file paths
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            if let Ok(name_str) = std::str::from_utf8(entry.name_bytes()) {
                let full_path = if path.is_empty() {
                    name_str.to_string()
                } else {
                    format!("{}/{}", path, name_str)
                };
                
                // Use lowercase version as key for case-insensitive comparison
                let lowercase_path = full_path.to_lowercase();
                path_map.entry(lowercase_path).or_default().push(full_path);
            }
        }
        git2::TreeWalkResult::Ok
    })?;
    
    // Find entries with more than one variant
    let mut duplicates = Vec::new();
    for (_, paths) in path_map {
        if paths.len() > 1 {
            duplicates.push(paths);
        }
    }
    
    // Sort for consistent output
    duplicates.sort();
    
    Ok(duplicates)
}