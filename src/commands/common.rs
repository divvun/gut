use anyhow::{Context, Result};

use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};

use crate::filter::{Filter, Filterable};
use crate::user::User;

pub fn query_and_filter_repositories(
    org: &str,
    regex: &Option<Filter>,
    token: &str,
) -> Result<Vec<RemoteRepo>> {
    let remote_repos = get_remote_repos(token, org)?;

    Ok(RemoteRepo::filter_with_option(remote_repos, regex))
}

pub fn get_user() -> Result<User> {
    User::get_user()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

pub fn get_user_token() -> Result<String> {
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
