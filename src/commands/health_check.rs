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
    repo: String,
    file_path: String,
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

        let mut total_repos = 0;
        let mut total_issues = 0;
        let mut all_issues = Vec::new();

        for owner in &owners {
            let owner_path = Path::new(&root).join(owner);
            if !owner_path.exists() {
                continue;
            }

            let repos = std::fs::read_dir(&owner_path)
                .with_context(|| format!("Cannot read directory {:?}", owner_path))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect::<Vec<_>>();

            total_repos += repos.len();

            println!("\n{} Checking {} repositories in {}...", 
                "→".cyan(), 
                repos.len(), 
                owner.bold()
            );

            for repo_path in repos {
                let repo_name = path::dir_name(&repo_path).unwrap_or_default();
                
                // Try to open as git repo
                let git_repo = match git::open(&repo_path) {
                    Ok(r) => r,
                    Err(_) => continue, // Skip non-git directories
                };

                let issues = check_repo_for_nfc_issues(&git_repo, &repo_path)?;
                
                if !issues.is_empty() {
                    println!("  {} {} ({} files with NFD normalization)", 
                        "✗".red(),
                        repo_name.yellow(),
                        issues.len()
                    );
                    total_issues += issues.len();
                    
                    for file_path in issues {
                        all_issues.push(NormalizationIssue {
                            repo: format!("{}/{}", owner, repo_name),
                            file_path,
                        });
                    }
                }
            }
        }

        // Print summary
        println!("\n{}", "═".repeat(80));
        if total_issues == 0 {
            println!("{} No NFD/NFC normalization issues found in {} repositories!", 
                "✓".green().bold(),
                total_repos
            );
        } else {
            println!("{} Found {} files with NFD normalization in {} repositories", 
                "⚠".yellow().bold(),
                total_issues,
                total_repos
            );
            
            // Print detailed table
            println!("\n{}", "Detailed list of affected files:".bold());
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
            table.set_titles(row!["Repository", "File Path"]);
            
            for issue in &all_issues {
                table.add_row(row![
                    cell!(b -> &issue.repo),
                    cell!(&issue.file_path)
                ]);
            }
            
            table.printstd();
            
            println!("\n{}", "Recommendation:".bold());
            println!("  1. Ensure {} is set on macOS:", "git config --global core.precomposeUnicode true".cyan());
            println!("     {}", "git config --global core.precomposeUnicode true".cyan());
            println!("  2. Use a tool like {} to fix affected repositories", "jaso".cyan());
            println!("  3. Consider creating a new commit with normalized filenames");
        }
        println!("{}", "═".repeat(80));

        Ok(())
    }
}

/// Check a single repository for NFC normalization issues
fn check_repo_for_nfc_issues(git_repo: &git2::Repository, _repo_path: &PathBuf) -> Result<Vec<String>> {
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
