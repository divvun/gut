use super::common;
use crate::github;

use crate::github::RemoteRepo;
use anyhow::Result;
use std::str;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Delete all web hooks for all repository that match regex
pub struct DeleteArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
}

impl DeleteArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        for repo in filtered_repos {
            let result = delete_all_hooks(&repo, &user_token);

            match result {
                Ok(n) => println!("Successful deleted {} hook(s) of repo {}", n, repo.name),
                Err(e) => println!(
                    "Failed to delete hook(s) on repo {} because {:?}",
                    repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn delete_all_hooks(repo: &RemoteRepo, token: &str) -> Result<usize> {
    let hooks = github::get_hooks(repo, token)?;
    let result: Vec<_> = hooks
        .iter()
        .map(|id| github::delete_hook(repo, *id, token))
        .collect();
    let result: Result<Vec<_>> = result.into_iter().collect();
    match result {
        Ok(_) => Ok(hooks.len()),
        Err(e) => Err(e),
    }
}
