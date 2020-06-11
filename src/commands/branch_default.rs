use super::common;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set a branch as default for all repositories that match a pattern
pub struct DefaultBranchArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// Name of the branch
    pub default_branch: String,
}

impl DefaultBranchArgs {
    pub fn set_default_branch(&self) -> Result<()> {
        let token = common::user_token()?;
        let repos =
            common::query_and_filter_repositories(&self.organisation, self.regex.as_ref(), &token)?;

        for repo in repos {
            let result = set_default_branch(&repo, &self.default_branch, &token);
            match result {
                Ok(_) => println!(
                    "Set default branch {} for repo {} successfully",
                    self.default_branch, repo.name
                ),
                Err(e) => println!(
                    "Could not set default branch {} for repo {} because {}",
                    self.default_branch, repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn set_default_branch(repo: &RemoteRepo, default_branch: &str, token: &str) -> Result<()> {
    github::set_default_branch(repo, default_branch, token)
}
