use super::common;
use crate::cli::Args as CommonArgs;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use clap::Parser;
use rayon::prelude::*;
use prettytable::{Table, format, row};

#[derive(Debug, Parser)]
/// Set a branch as protected for all local repositories that match a pattern
pub struct ProtectedBranchArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Name of the branch
    pub protected_branch: String,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl ProtectedBranchArgs {
    pub fn set_protected_branch(&self, _common_args: &CommonArgs) -> Result<()> {
        if self.all_orgs {
            let organizations = common::get_all_organizations()?;
            if organizations.is_empty() {
                println!("No organizations found in root directory");
                return Ok(());
            }
            
            let mut summaries = Vec::new();
            
            for org in &organizations {
                println!("\n=== Processing organization: {} ===", org);
                
                match self.run_for_organization(org) {
                    Ok(summary) => {
                        summaries.push(summary);
                    },
                    Err(e) => {
                        println!("Failed to process organization '{}': {:?}", org, e);
                    }
                }
            }
            
            print_protect_branch_summary(&summaries);
            
            Ok(())
        } else {
            let organisation = common::organisation(self.organisation.as_deref())?;
            self.run_for_organization(&organisation)?;
            Ok(())
        }
    }

    fn run_for_organization(&self, organisation: &str) -> Result<common::OrgResult> {
        let user_token = common::user_token()?;
        let filtered_repos =
            common::query_and_filter_repositories(organisation, self.regex.as_ref(), &user_token)?;

        let mut result = common::OrgResult::new(organisation.to_string());

        // Process repos and track results
        for repo in filtered_repos.iter() {
            let protect_result = set_protected_branch(repo, &self.protected_branch, &user_token);
            match protect_result {
                Ok(_) => {
                    println!(
                        "Set protected branch {} for repo {} successfully",
                        self.protected_branch, repo.name
                    );
                    result.add_success();
                },
                Err(e) => {
                    println!(
                        "Could not set protected branch {} for repo {} because of {}",
                        self.protected_branch, repo.name, e
                    );
                    result.add_failure();
                },
            }
        }

        Ok(result)
    }
}

fn set_protected_branch(repo: &RemoteRepo, protected_branch: &str, token: &str) -> Result<()> {
    github::set_protected_branch(repo, protected_branch, token)
}

fn print_protect_branch_summary(summaries: &[common::OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Protected", "Failed"]);

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
    }

    println!("\n=== All org summary ===");
    table.printstd();
}
