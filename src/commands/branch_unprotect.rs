use super::common::{self, OrgResult};
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use clap::Parser;

#[derive(Debug, Parser)]
/// Remove branch protection for all local repositories that match a pattern
pub struct UnprotectedBranchArgs {
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
    pub branch: String,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl UnprotectedBranchArgs {
    pub fn set_unprotected_branch(&self) -> Result<()> {
        common::run_for_orgs(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org),
            "Unprotected",
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<OrgResult> {
        let user_token = common::user_token()?;
        let filtered_repos =
            common::query_and_filter_repositories(organisation, self.regex.as_ref(), &user_token)?;

        let mut result = OrgResult::new(organisation);

        // Process repos and track results
        for repo in filtered_repos.iter() {
            let unprotect_result = set_unprotected_branch(repo, &self.branch, &user_token);
            match unprotect_result {
                Ok(_) => {
                    println!(
                        "Removed protection on branch {} for repo {} successfully",
                        self.branch, repo.name
                    );
                    result.add_success();
                }
                Err(e) => {
                    println!(
                        "Could not remove protection on branch {} for repo {} because of {}",
                        self.branch, repo.name, e
                    );
                    result.add_failure();
                }
            }
        }

        Ok(result)
    }
}

fn set_unprotected_branch(repo: &RemoteRepo, branch: &str, token: &str) -> Result<()> {
    github::set_unprotected_branch(repo, branch, token)
}
