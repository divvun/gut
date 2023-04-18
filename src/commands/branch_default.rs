use super::common;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use clap::Parser;

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
}

impl DefaultBranchArgs {
    pub fn set_default_branch(&self) -> Result<()> {
        let token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &token)?;

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
