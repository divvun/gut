use super::common;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set a branch as protected for all local repositories that match a pattern
pub struct ProtectedBranchArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// Name of the branch
    pub protected_branch: String,
}

impl ProtectedBranchArgs {
    pub fn set_protected_branch(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user_token,
        )?;

        for repo in filtered_repos {
            let result = set_protected_branch(&repo, &self.protected_branch, &user_token);
            match result {
                Ok(_) => println!(
                    "Set protected branch {} for repo {} successfully",
                    self.protected_branch, repo.name
                ),
                Err(e) => println!(
                    "Could not set protected branch {} for repo {} because of {}",
                    self.protected_branch, repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn set_protected_branch(repo: &RemoteRepo, protected_branch: &str, token: &str) -> Result<()> {
    github::set_protected_branch(repo, protected_branch, token)
}
