use crate::cli::Args as CommonArgs;
use super::common;
use crate::github;

use crate::github::RemoteRepo;
use anyhow::Result;
use std::str;

use crate::filter::Filter;
use clap::Parser;

#[derive(Debug, Parser)]
/// Delete ALL web hooks for all repositories that match given regex
pub struct DeleteArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
}

impl DeleteArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, Some(&self.regex), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        for repo in filtered_repos {
            let result = delete_all_hooks(&repo, &user_token);

            match result {
                Ok(n) => println!("Successfully deleted {} hook(s) of repo {}", n, repo.name),
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
    let result = hooks.iter().map(|id| github::delete_hook(repo, *id, token));
    let result: Result<Vec<_>> = result.into_iter().collect();
    match result {
        Ok(_) => Ok(hooks.len()),
        Err(e) => Err(e),
    }
}
