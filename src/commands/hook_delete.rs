use super::common;
use crate::github;

use crate::github::RemoteRepo;
use anyhow::Result;
use std::str;

use crate::filter::Filter;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Delete ALL web hooks for all repositories that match given regex
pub struct DeleteArgs {
    #[arg(long, short)]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
}

impl DeleteArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, Some(&self.regex), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in owner {} that match the pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        filtered_repos.par_iter().for_each(|repo| {
            let result = delete_all_hooks(repo, &user_token);

            match result {
                Ok(n) => println!("Successfully deleted {} hook(s) of repo {}", n, repo.name),
                Err(e) => println!(
                    "Failed to delete hook(s) on repo {} because {:?}",
                    repo.name, e
                ),
            }
        });

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
