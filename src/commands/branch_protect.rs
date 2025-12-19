use super::common;
use crate::cli::Args as CommonArgs;
use crate::github;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use clap::Parser;
use rayon::prelude::*;

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
    #[arg(short = 'A', long = "all-orgs")]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl ProtectedBranchArgs {
    pub fn set_protected_branch(&self, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        filtered_repos.par_iter().for_each(|repo| {
            let result = set_protected_branch(repo, &self.protected_branch, &user_token);
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
        });

        Ok(())
    }
}

fn set_protected_branch(repo: &RemoteRepo, protected_branch: &str, token: &str) -> Result<()> {
    github::set_protected_branch(repo, protected_branch, token)
}
