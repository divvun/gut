use crate::config::Config;
use crate::path;
use anyhow::{anyhow, Context, Result};
use dialoguer::Input;
use std::path::PathBuf;

use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};

use crate::filter::{Filter, Filterable};
use crate::user::User;

pub fn query_and_filter_repositories(
    org: &str,
    regex: Option<&Filter>,
    token: &str,
) -> Result<Vec<RemoteRepo>> {
    let remote_repos = remote_repos(token, org)?;

    Ok(RemoteRepo::filter_with_option(remote_repos, regex))
}

pub fn user() -> Result<User> {
    User::user()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

pub fn root() -> Result<String> {
    Config::root()
        .context("Cannot read the config file. Run dadmin init with valid token and root directory")
}

pub fn user_token() -> Result<String> {
    User::token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match github::list_org_repos(token, org).context("When fetching repositories") {
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

pub fn read_dirs_for_org(org: &str, root: &str, filter: Option<&Filter>) -> Result<Vec<PathBuf>> {
    let target_dir = path::local_path_org(org, &root)?;

    let result = match filter {
        Some(f) => read_dirs_with_filter(&target_dir, &f),
        None => read_dirs(&target_dir),
    };

    match result {
        Ok(r) => Ok(r),
        Err(e) => Err(anyhow!(
            "Cannot read sub directories for organisation {} \"{}\" because {:?}",
            target_dir.display(),
            org,
            e
        )),
    }
}

/// Filter directory's name by regex
pub fn read_dirs_with_filter(path: &PathBuf, filter: &Filter) -> Result<Vec<PathBuf>> {
    let dirs = read_dirs(path)?;
    Ok(PathBuf::filter(dirs, filter))
}

/// Read all dirs inside a path
/// Filter directories
fn read_dirs(path: &PathBuf) -> Result<Vec<PathBuf>> {
    let entries = path.read_dir()?;
    let dirs = entries
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|x| x.is_dir())
        .collect();
    Ok(dirs)
}

pub fn confirm(prompt: &str, key: &str) -> Result<bool> {
    let confirm = Input::<String>::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact()?;
    Ok(confirm == key)
}
