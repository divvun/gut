use super::common;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Remove branch protection for all local repositories that match a pattern
pub struct UnprotectedBranchArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// Name of the branch
    pub branch: String,
}

impl UnprotectedBranchArgs {
    pub fn set_unprotected_branch(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        for repo in filtered_repos {
            let result = set_unprotected_branch(&repo, &self.branch, &user_token);
            match result {
                Ok(_) => println!(
                    "Removed protection on branch {} for repo {} successfully",
                    self.branch, repo.name
                ),
                Err(e) => println!(
                    "Could not remove protection on branch {} for repo {} because of {}",
                    self.branch, repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn set_unprotected_branch(repo: &RemoteRepo, branch: &str, token: &str) -> Result<()> {
    github::set_unprotected_branch(repo, branch, token)
}
