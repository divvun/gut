use crate::api::{list_org_repos, RemoteRepo};

use anyhow::{Context, Result};

use crate::filter::{Filter, Filterable};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct DefaultBranchArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub default_branch: String,
}

impl DefaultBranchArgs {
    pub fn set_default_branch(&self) -> Result<()> {
        let user_token = get_user_token()?;

        let remote_repos = get_remote_repos(&user_token, &self.organisation)?;

        let filtered_repos = RemoteRepo::filter_with_option(remote_repos, &self.regex);
        for repo in filtered_repos {
            let result = set_default_branch(&repo, &self.default_branch, &user_token);
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
    Ok(())
}

fn get_user_token() -> Result<String> {
    super::User::get_token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn get_remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match list_org_repos(token, org).context("Fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<crate::api::NoReposFound>() {
                anyhow::bail!("No repositories found");
            }
            if let Some(_) = e.downcast_ref::<crate::api::Unauthorized>() {
                anyhow::bail!("User token invalid. Run dadmin init with a valid token");
            }
            return Err(e);
        }
    }
}
