use anyhow::{Context, Result};

use crate::filter::{Filter, Filterable};
use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ListRepoArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl ListRepoArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = get_user_token()?;

        let remote_repos = get_remote_repos(&user_token, &self.organisation)?;

        let filtered_repos = RemoteRepo::filter_with_option(remote_repos, &self.regex);

        print_results(&filtered_repos);

        Ok(())
    }
}

fn get_user_token() -> Result<String> {
    super::User::get_token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn get_remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match github::list_org_repos(token, org).context("Fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<NoReposFound>() {
                anyhow::bail!("No repositories found");
            }
            if let Some(_) = e.downcast_ref::<Unauthorized>() {
                anyhow::bail!("User token invalid. Run dadmin init with a valid token");
            }
            Err(e)
        }
    }
}

fn print_results(repos: &Vec<RemoteRepo>) {
    for repo in repos {
        println!("{:?}", repo);
    }
}
