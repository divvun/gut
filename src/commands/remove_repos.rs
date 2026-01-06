use super::common;

use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct RemoveReposArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
}

impl RemoveReposArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in organisation {} that match the pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        let is_confirmed = confirm(&filtered_repos)?;
        if is_confirmed {
            remove(&filtered_repos, &user_token)?;
        } else {
            println!("Command is aborted. Nothing got deleted!")
        }
        Ok(())
    }
}

fn confirm(repos: &[RemoteRepo]) -> Result<bool> {
    println!("The following repos will be removed:");

    for repo in repos {
        println!("{}", repo.full_name());
    }

    let key = "YES";
    common::confirm(
        &format!(
            "Are you sure you want to delete {} repo(s)?\nEnter {} to continue",
            repos.len(),
            key
        ),
        key,
    )
}

fn remove(repos: &[RemoteRepo], token: &str) -> Result<()> {
    for repo in repos {
        match github::delete_repo(&repo.owner, &repo.name, token) {
            Ok(_) => println!("Deleted repo {} successfully", repo.full_name()),
            Err(e) => println!("Failed to delete repo {} because {:?}", repo.full_name(), e),
        }
    }
    Ok(())
}
