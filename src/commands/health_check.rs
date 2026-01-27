use super::common;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use prettytable::{Table, cell, format, row};
use std::path::{Path, PathBuf};
use unicode_normalization::is_nfc;

#[derive(Debug, Parser)]
/// Check for NFD/NFC normalization conflicts in repository filenames
///
/// This command scans all files in local repositories and reports any filenames
/// that contain Unicode decomposed (NFD) characters that could cause conflicts
/// with Git's expected NFC normalization on macOS.
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

struct RepoCheckResult {
    repo_name: String,
    issues: Vec<String>,
}

struct OwnerSummary {
    owner: String,
    total_repos: usize,
    issues: Vec<NormalizationIssue>,
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

        for owner in &owners {
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
        let results = common::process_with_progress(
            "Checking",
            &repos,
            |repo_path| check_repo(repo_path),
            |result| result.repo_name.clone(),
        );

        // Collect all issues
        let mut all_issues = Vec::new();
        for result in results {
            for file_path in result.issues {
                all_issues.push(NormalizationIssue {
                    owner: owner.to_string(),
                    repo: result.repo_name.clone(),
                    file_path,
                });
            }
        }

        Ok(OwnerSummary {
            owner: owner.to_string(),
            total_repos,
            issues: all_issues,
        })
    }

    fn print_owner_summary(&self, summary: &OwnerSummary) {
        println!("\n{} {}:", "Owner:".bold(), summary.owner.cyan().bold());
        
        if summary.issues.is_empty() {
            println!("  {} All filenames are correctly encoded", "✓".green().bold());
        } else {
            let repo_count = summary.issues.iter()
                .map(|i| i.repo.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len();
            
            println!("  {} Found {} files with NFD normalization in {} repositories", 
                "⚠".yellow().bold(),
                summary.issues.len(),
                repo_count
            );
            
            // Group by repo
            let mut by_repo: std::collections::HashMap<String, Vec<&NormalizationIssue>> = 
                std::collections::HashMap::new();
            
            for issue in &summary.issues {
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
    }

    fn print_single_owner_summary(&self, summary: &OwnerSummary) {
        println!("\n{}", "═".repeat(80));
        if summary.issues.is_empty() {
            println!("{} All filenames are correctly encoded in {} repositories!", 
                "✓".green().bold(),
                summary.total_repos
            );
        } else {
            let repo_count = summary.issues.iter()
                .map(|i| i.repo.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len();
            
            println!("{} Found {} files with NFD normalization in {} of {} repositories", 
                "⚠".yellow().bold(),
                summary.issues.len(),
                repo_count,
                summary.total_repos
            );
            
            // Print detailed table
            println!("\n{}", "Detailed list of affected files:".bold());
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
            table.set_titles(row!["Repository", "File Path"]);
            
            for issue in &summary.issues {
                table.add_row(row![
                    cell!(b -> &issue.repo),
                    cell!(&issue.file_path)
                ]);
            }
            
            table.printstd();
            
            self.print_recommendations();
        }
        println!("{}", "═".repeat(80));
    }

    fn print_final_summary(&self, summaries: &[OwnerSummary]) {
        println!("\n{}", "═".repeat(80));
        println!("{}", "FINAL SUMMARY".bold());
        println!("{}", "═".repeat(80));
        
        let total_repos: usize = summaries.iter().map(|s| s.total_repos).sum();
        let total_issues: usize = summaries.iter().map(|s| s.issues.len()).sum();
        
        if total_issues == 0 {
            println!("{} All filenames are correctly encoded in {} repositories across {} owners!", 
                "✓".green().bold(),
                total_repos,
                summaries.len()
            );
        } else {
            println!("{} Found {} files with NFD normalization across {} owners", 
                "⚠".yellow().bold(),
                total_issues,
                summaries.len()
            );
            
            // Collect all issues
            let mut all_issues: Vec<&NormalizationIssue> = summaries.iter()
                .flat_map(|s| s.issues.iter())
                .collect();
            
            all_issues.sort_by(|a, b| {
                a.owner.cmp(&b.owner).then(a.repo.cmp(&b.repo))
            });
            
            // Print detailed table with separate columns
            println!("\n{}", "Detailed list of affected files:".bold());
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
            table.set_titles(row!["Owner", "Repository", "File Path"]);
            
            for issue in all_issues {
                table.add_row(row![
                    cell!(b -> &issue.owner),
                    cell!(b -> &issue.repo),
                    cell!(&issue.file_path)
                ]);
            }
            
            table.printstd();
            
            self.print_recommendations();
        }
        println!("{}", "═".repeat(80));
    }

    fn print_recommendations(&self) {
        println!("\n{}", "Recommendations:".bold());
        println!("  1. Ensure {} is set on macOS:", "git config --global core.precomposeUnicode true".cyan());
        println!("     {}", "git config --global core.precomposeUnicode true".cyan());
        println!("  2. Use a tool like {} to fix affected repositories", "jaso".cyan());
        println!("  3. Consider creating a new commit with normalized filenames");
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
            issues: Vec::new(),
        },
    };

    let issues = check_repo_for_nfc_issues(&git_repo).unwrap_or_default();
    
    RepoCheckResult {
        repo_name,
        issues,
    }
}

/// Check a single repository for NFC normalization issues
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
            let full_path = if path.is_empty() {
                entry.name().unwrap_or("").to_string()
            } else {
                format!("{}/{}", path, entry.name().unwrap_or(""))
            };
            
            // Check if the path contains NFD characters
            if !is_nfc(&full_path) {
                issues.push(full_path);
            }
        }
        git2::TreeWalkResult::Ok
    })?;
    
    Ok(issues)
}
