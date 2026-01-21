use super::common::{self, OrgResult};
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use clap::Parser;

#[derive(Debug, Parser)]
/// Set a branch as protected for all local repositories that match a pattern
pub struct ProtectedBranchArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Name of the branch
    pub protected_branch: String,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

impl ProtectedBranchArgs {
    pub fn set_protected_branch(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Protected",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        let user_token = common::user_token()?;
        let filtered_repos =
            common::query_and_filter_repositories(owner, self.regex.as_ref(), &user_token)?;

        let mut result = OrgResult::new(owner);

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
