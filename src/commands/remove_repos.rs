use super::common;

use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RemoveReposArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl RemoveReposArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, &self.regex, &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        let is_confirmed = confirm(&filtered_repos)?;
        if is_confirmed {
            remove(&filtered_repos, &user_token)?;
        } else {
            println!("Nothing got deleted!")
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
            "Are you sure you want to delete {} repo(s)? Enter {} to continue: ",
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
