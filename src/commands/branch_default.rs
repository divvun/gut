use super::common;
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use clap::Parser;
use prettytable::{Table, format, row};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Set a branch as default for all repositories that match a pattern
pub struct DefaultBranchArgs {
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
    pub default_branch: String,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl DefaultBranchArgs {
    pub fn set_default_branch(&self, _common_args: &CommonArgs) -> Result<()> {
        common::run_for_orgs_or_single(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org),
            Some(print_default_branch_summary),
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<common::OrgResult> {
        let token = common::user_token()?;
        let repos =
            common::query_and_filter_repositories(organisation, self.regex.as_ref(), &token)?;

        let mut result = common::OrgResult::new(organisation.to_string());

        // Process repos and track results
        for repo in repos.iter() {
            let set_result = set_default_branch(repo, &self.default_branch, &token);
            match set_result {
                Ok(_) => {
                    println!(
                        "Set default branch {} for repo {} successfully",
                        self.default_branch, repo.name
                    );
                    result.add_success();
                }
                Err(e) => {
                    println!(
                        "Could not set default branch {} for repo {} because {}",
                        self.default_branch, repo.name, e
                    );
                    result.add_failure();
                }
            }
        }

        Ok(result)
    }
}

fn set_default_branch(repo: &RemoteRepo, default_branch: &str, token: &str) -> Result<()> {
    github::set_default_branch(repo, default_branch, token)
}

fn print_default_branch_summary(summaries: &[common::OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Default Set", "Failed"]);

    let mut total_repos = 0;
    let mut total_set = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_set += summary.successful_repos;
        total_failed += summary.failed_repos;
    }
    table.add_empty_row();
    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_set,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
