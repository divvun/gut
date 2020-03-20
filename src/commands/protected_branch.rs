use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};

use crate::user::User;
use anyhow::{Context, Result};

use crate::filter::{Filter, Filterable};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ProtectedBranchArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub protected_branch: String,
}

impl ProtectedBranchArgs {
    pub fn set_protected_branch(&self) -> Result<()> {
        let user_token = get_user_token()?;

        let remote_repos = get_remote_repos(&user_token, &self.organisation)?;

        let filtered_repos = RemoteRepo::filter_with_option(remote_repos, &self.regex);

        for repo in filtered_repos {
            let result = set_protected_branch(&repo, &self.protected_branch, &user_token);
            match result {
                Ok(_) => println!(
                    "Set protected branch {} for repo {} successfully",
                    self.protected_branch, repo.name
                ),
                Err(e) => println!(
                    "Could not set protected branch {} for repo {} because {}",
                    self.protected_branch, repo.name, e
                ),
            }
        }
        Ok(())
    }
}

fn set_protected_branch(repo: &RemoteRepo, protected_branch: &str, token: &str) -> Result<()> {
    github::set_protected_branch(repo, protected_branch, token)
}

fn get_user_token() -> Result<String> {
    User::get_token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn get_remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match github::list_org_repos(token, org).context("Fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if e.downcast_ref::<NoReposFound>().is_some() {
                anyhow::bail!("No repositories found");
            }
            if e.downcast_ref::<Unauthorized>().is_some() {
                anyhow::bail!("User token invalid. Run dadmin init with a valid token");
            }
            Err(e)
        }
    }
}
