use super::common;
use crate::github::create_org_repo;
use crate::user::User;
use std::path::PathBuf;

use super::models::Directory;
use crate::path;
use anyhow::{anyhow, Context, Result};

use crate::filter::Filter;
use crate::git::{GitCredential, open, push};
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
    #[structopt(long)]
    pub override_origin: bool,
    #[structopt(long)]
    pub clone: bool,
}

impl CreateRepoArgs {
    pub fn create_repo(&self) -> Result<()> {
        log::debug!("Create Repo {:?}", self);

        let dir = match &self.dir {
            Some(d) => d.path.clone(),
            None => path::local_path_org(&self.organisation)?,
        };

        let sub_dirs = common::read_dirs_with_filter(&dir, &self.regex)?;

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
                self.override_origin,
            ) {
                Ok(repo) => println!("Created repo for {} successfully at: {}", repo.name, repo.url),
                Err(e) => println!("Failed to create repo for dir: {:?} because {:?}", &dir, e),
            }
        }
        Ok(())
    }
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
    override_remote: bool,
) -> Result<CreateRepo> {
    let git_repo = open::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    if git_repo.find_remote(remote_name).is_ok() {
        if override_remote {
            git_repo.remote_delete(remote_name)?;
        } else {
            return Err(anyhow!(
                "This repo already has a remote named: {}",
                remote_name
            ));
        }
    }

    let branches: Vec<String> = git_repo
        .branches(Some(git2::BranchType::Local))
        .unwrap()
        .map(|a| a.unwrap())
        .map(|(a, _)| a.name().unwrap().unwrap().to_string())
        .collect();

    if branches.is_empty() {
        return Err(anyhow!("This repository doesn't have any local branch"));
    }

    // todo path::dir_name
    let repo_name = dir
        .file_name()
        .ok_or_else(|| anyhow!("{:?} does not have a vaild name"))?
        .to_str()
        .ok_or_else(|| anyhow!("{:?} doesn not have a valid name", dir))?;

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

    let create_repo = CreateRepo {
        name: repo_name.to_string(),
        url: created_repo.html_url,
    };

    Ok(create_repo)
}

#[derive(Debug)]
struct CreateRepo {
    name: String,
    url: String,
}

// clone after create
// this may need a clone common function
// for both clone and create command
