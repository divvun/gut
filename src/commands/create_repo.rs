use super::common;
use crate::github::create_org_repo;
use crate::user::User;
use std::path::PathBuf;

use crate::path::{local_path_org, Directory};
use anyhow::{anyhow, Context, Result};

use crate::filter::{Filter, Filterable};
use crate::git::open;
use crate::git::push;
use crate::git::GitCredential;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateRepoArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Filter,
    #[structopt(long, short)]
    pub dir: Option<Directory>,
    #[structopt(long, short)]
    pub public: bool,
    #[structopt(long, short)]
    pub use_https: bool,
    #[structopt(long, short)]
    pub no_push: bool,
}

impl CreateRepoArgs {
    pub fn create_repo(&self) -> Result<()> {
        log::debug!("Create Repo {:?}", self);

        let dir = match &self.dir {
            Some(d) => d.path.clone(),
            None => local_path_org(&self.organisation)?,
        };

        let sub_dirs = read_dirs(&dir, &self.regex)?;

        log::debug!("Filtered sub dirs: {:?}", sub_dirs);
        let user = common::user()?;
        for dir in sub_dirs {
            match create_repo(
                &self.organisation,
                &dir,
                self.public,
                &user,
                &"origin",
                self.use_https,
                self.no_push,
            ) {
                Ok((name, url)) => println!("Created repo for {} successfully at: {}", name, url),
                Err(e) => println!("Failed to create repo for dir: {:?} because {:?}", &dir, e),
            }
        }
        Ok(())
    }
}

/// Read all dirs inside a path
/// Filter directories
/// Filter directory's name by regex

fn read_dirs(path: &PathBuf, filter: &Filter) -> Result<Vec<PathBuf>> {
    let entries = path.read_dir()?;
    let dirs = entries
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|x| x.is_dir())
        .collect();
    Ok(PathBuf::filter(dirs, filter))
}

/// Check if {dir} is a git repository
/// Check if {dir} has {remote} remote
/// Create a new repository in organization {org}
/// Set the new created repository as remote {remote}
/// Push all to remote {remote}

fn create_repo(
    org: &str,
    dir: &PathBuf,
    public: bool,
    user: &User,
    remote_name: &str,
    use_https: bool,
    no_push: bool,
) -> Result<(String, String)> {
    let git_repo = open::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    if git_repo.find_remote(remote_name).is_ok() {
        return Err(anyhow!(
            "This repo already has a remote named: {}",
            remote_name
        ));
    }

    let repo_name = dir
        .file_name()
        .ok_or_else(|| anyhow!("{:?} does not have a vaild name"))?
        .to_str()
        .ok_or_else(|| anyhow!("{:?} doesn not have a valid name", dir))?;

    let branches: Vec<String> = git_repo
        .branches(Some(git2::BranchType::Local))
        .unwrap()
        .map(|a| a.unwrap())
        .map(|(a, _)| a.name().unwrap().unwrap().to_string())
        .collect();

    if branches.is_empty() {
        return Err(anyhow!("This repository doesn't have any local branch"));
    }

    let created_repo = create_org_repo(org, repo_name, public, &user.token)?;
    log::debug!("new created repo: {:?}", created_repo.html_url);

    let remote_url = if use_https {
        format!("{}.git", created_repo.html_url.clone())
    } else {
        created_repo.ssh_url.clone()
    };

    let mut remote = git_repo.remote(remote_name, &remote_url)?;

    if !no_push {
        let cred = GitCredential::from(user);
        push::push(&git_repo, &mut remote, Some(cred))?;
    }

    Ok((repo_name.to_string(), created_repo.html_url))
}
