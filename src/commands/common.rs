use crate::config::Config;
use crate::path;
use anyhow::{anyhow, Context, Result};
use dialoguer::Input;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    let mut result = RemoteRepo::filter_with_option(remote_repos, regex);
    result.sort();
    Ok(result)
}

pub fn user() -> Result<User> {
    User::from_config()
        .context("Cannot get user token from the config file. Run `gut init` with a valid token")
}

pub fn root() -> Result<String> {
    Config::root()
        .context("Cannot read the config file. Run `gut init` with valid token and root directory")
}

pub fn user_token() -> Result<String> {
    User::token()
        .context("Cannot get user token from the config file. Run `gut init` with a valid token")
}

pub fn organisation(opt: Option<&str>) -> Result<String> {
    match opt {
        Some(s) => Ok(s.to_string()),
        None => {
            let config = Config::from_file()?;
            match config.default_org {
                Some(o) => Ok(o),
                None => anyhow::bail!("You need to provide an organisation or set a default organisation with init/set default organisation command."),
            }
        }
    }
}

fn remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match github::list_org_repos(token, org).context("When fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if e.downcast_ref::<NoReposFound>().is_some() {
                anyhow::bail!("No repositories found");
            }
            if e.downcast_ref::<Unauthorized>().is_some() {
                anyhow::bail!("User token invalid. Run `gut init` with a valid token");
            }
            Err(e)
        }
    }
}

pub fn read_dirs_for_org(org: &str, root: &str, filter: Option<&Filter>) -> Result<Vec<PathBuf>> {
    let target_dir = path::local_path_org(org, root)?;

    let result = match filter {
        Some(f) => read_dirs_with_filter(&target_dir, f),
        None => read_dirs(&target_dir),
    };

    match result {
        Ok(mut vec) => {
            vec.sort();
            Ok(vec)
        }
        Err(e) => Err(anyhow!(
            "Cannot read sub directories for organisation {} \"{}\" because {:?}",
            target_dir.display(),
            org,
            e
        )),
    }
}

/// Filter directory's name by regex
pub fn read_dirs_with_filter(path: &Path, filter: &Filter) -> Result<Vec<PathBuf>> {
    let dirs = read_dirs(path)?;
    Ok(PathBuf::filter(dirs, filter))
}

/// Read all dirs inside a path
/// Filter directories
fn read_dirs(path: &Path) -> Result<Vec<PathBuf>> {
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

pub fn ask_for(prompt: &str) -> Result<String> {
    let confirm = Input::<String>::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact()?;
    Ok(confirm)
}

pub fn apply_script(dir: &PathBuf, script: &str) -> Result<Output> {
    let output = execute_script(script, dir)?;
    if output.status.success() {
        Ok(output)
    } else {
        let err_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| format!("Cannot execute the script {}", script));
        Err(anyhow!(err_message))
    }
}

fn execute_script(script: &str, dir: &PathBuf) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", script])
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    };

    //log::debug!("Script result {:?}", output);

    Ok(output)
}

pub fn sub_strings(string: &str, sub_len: usize) -> Vec<&str> {
    let mut subs = Vec::with_capacity(string.len() / sub_len);
    let mut iter = string.chars();
    let mut pos = 0;

    while pos < string.len() {
        let mut len = 0;
        for ch in iter.by_ref().take(sub_len) {
            len += ch.len_utf8();
        }
        subs.push(&string[pos..pos + len]);
        pos += len;
    }
    subs
}
