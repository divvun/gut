use anyhow::{Context, Result};
use dialoguer::Confirmation;
use std::path::PathBuf;

use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};

use crate::filter::{Filter, Filterable};
use crate::user::User;

pub fn query_and_filter_repositories(
    org: &str,
    regex: &Option<Filter>,
    token: &str,
) -> Result<Vec<RemoteRepo>> {
    let remote_repos = remote_repos(token, org)?;

    Ok(RemoteRepo::filter_with_option(remote_repos, regex))
}

pub fn user() -> Result<User> {
    User::user()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

pub fn user_token() -> Result<String> {
    User::token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
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

/// Read all dirs inside a path
/// Filter directories
/// Filter directory's name by regex
pub fn read_dirs(path: &PathBuf, filter: &Filter) -> Result<Vec<PathBuf>> {
    let entries = path.read_dir()?;
    let dirs = entries
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|x| x.is_dir())
        .collect();
    Ok(PathBuf::filter(dirs, filter))
}

pub fn confirm(prompt: &str) -> Result<bool> {
    let confirm = Confirmation::new().with_text(prompt).interact()?;
    Ok(confirm)
}
