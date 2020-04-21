use super::common;
use crate::github::create_org_repo;
use crate::user::User;
use std::path::PathBuf;

use super::models::Directory;
use crate::path;
use anyhow::{anyhow, Context, Result};

use crate::filter::Filter;
use crate::git::{open, push, Clonable, GitCredential, GitRepo};
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
    pub fn run(&self) -> Result<()> {
        log::debug!("Create Repo {:?}", self);

        let dir = match &self.dir {
            Some(d) => d.path.clone(),
            None => path::local_path_org(&self.organisation)?,
        };

        let sub_dirs = common::read_dirs_with_filter(&dir, &self.regex)?;

        log::debug!("Filtered sub dirs: {:?}", sub_dirs);

        let user = common::user()?;
        for dir in sub_dirs {
            create_and_clone(
                &self.organisation,
                &dir,
                self.public,
                &user,
                &"origin",
                self.use_https,
                self.no_push,
                self.override_origin,
                self.clone,
            );
        }
        Ok(())
    }
}

fn create_and_clone(
    org: &str,
    dir: &PathBuf,
    public: bool,
    user: &User,
    remote_name: &str,
    use_https: bool,
    no_push: bool,
    override_remote: bool,
    clone: bool,
) {
    match create_repo(
        org,
        dir,
        public,
        user,
        remote_name,
        use_https,
        no_push,
        override_remote,
    ) {
        Ok(created_repo) => {
            println!(
                "Created repo for {} successfully at: {}",
                created_repo.name, created_repo.html_url
            );
            if !clone {
                return;
            }

            match clone_repo(&created_repo, user, org, use_https) {
                Ok(gp) => {
                    println!("And then cloned at {:?}", gp.local_path);
                }
                Err(ce) => {
                    println!("Clone failed because of {}", ce);
                }
            }
        }
        Err(e) => {
            println!("Failed to create repo for dir: {:?} because {:?}", &dir, e);
        }
    }
}

fn clone_repo(repo: &CreateRepo, user: &User, org: &str, use_https: bool) -> Result<GitRepo> {
    let git_repo = repo.to_git_repo(org, user, use_https)?;

    let (gp, _) = git_repo.gclone()?;

    Ok(gp)
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
        html_url: created_repo.html_url,
        ssh_url: created_repo.ssh_url,
        https_url: created_repo.clone_url,
    };

    Ok(create_repo)
}

#[derive(Debug)]
struct CreateRepo {
    name: String,
    html_url: String,
    ssh_url: String,
    https_url: String,
}

impl CreateRepo {
    fn to_git_repo(&self, org: &str, user: &User, use_https: bool) -> Result<GitRepo> {
        let local_path = path::local_path_repo(org, &self.name)
            .ok_or_else(|| anyhow!("Cannot create local path for {}/{}", org, self.name))?;
        let remote_url = if use_https {
            self.https_url.to_string()
        } else {
            self.ssh_url.to_string()
        };

        let cred = GitCredential::from(user);

        Ok(GitRepo {
            remote_url,
            local_path,
            cred: Some(cred),
        })
    }
}
