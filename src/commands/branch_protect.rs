use super::common::{self, OrgResult};
use crate::cli::Args as CommonArgs;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use clap::Parser;
use prettytable::{Table, format, row};

#[derive(Debug, Parser)]
/// Set a branch as protected for all local repositories that match a pattern
pub struct ProtectedBranchArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
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
        common::run_for_orgs(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org),
            Some(print_protect_branch_summary),
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<OrgResult> {
        let user_token = common::user_token()?;
        let filtered_repos =
            common::query_and_filter_repositories(organisation, self.regex.as_ref(), &user_token)?;

        let mut result = OrgResult::new(organisation.to_string());

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
                }
                Err(e) => {
                    println!(
                        "Could not set protected branch {} for repo {} because of {}",
                        self.protected_branch, repo.name, e
                    );
                    result.add_failure();
                }
            }
        }

        Ok(result)
    }
}

fn set_protected_branch(repo: &RemoteRepo, protected_branch: &str, token: &str) -> Result<()> {
    github::set_protected_branch(repo, protected_branch, token)
}

fn print_protect_branch_summary(summaries: &[OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Protected", "Failed"]);

    let mut total_repos = 0;
    let mut total_protected = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_protected += summary.successful_repos;
        total_failed += summary.failed_repos;
    }

    table.add_empty_row();

    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_protected,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
