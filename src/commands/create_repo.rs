use super::common;
use crate::github::create_org_repo;
use crate::user::User;
use std::path::PathBuf;

use super::models::ExistDirectory;
use crate::path;
use anyhow::{anyhow, Context, Result};

use crate::filter::Filter;
use crate::git::{open, push, Clonable, GitCredential, GitRepo};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Create new repositories in an organisation and push for existing git repositories
pub struct CreateRepoArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// The parent directory of all directories that you want to create new repositories
    pub dir: Option<ExistDirectory>,
    #[structopt(long, short)]
    /// Regex to filter out sub directories by name
    pub regex: Filter,
    #[structopt(long, short)]
    /// Option to create a public repositories
    pub public: bool,
    #[structopt(long, short)]
    /// Option to not pushing the new created repositories to github
    pub no_push: bool,
    #[structopt(long)]
    /// Option to clone the new created repositories right after it is being created.
    pub clone: bool,
    #[structopt(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
    #[structopt(long)]
    /// Option to overrrite the exist remote origin
    pub override_origin: bool,
}

impl CreateRepoArgs {
    pub fn run(&self) -> Result<()> {
        log::debug!("Create Repo {:?}", self);

        let root = common::root()?;

        let sub_dirs = match &self.dir {
            Some(d) => common::read_dirs_with_filter(&d.path, &self.regex).with_context(|| {
                format!(
                    "Cannot read sub directories for \"{}\" because {:?}",
                    d.path.display(),
                    self
                )
            })?,
            None => common::read_dirs_for_org(&self.organisation, &root, Some(&self.regex))?,
        };

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
                &root,
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
    root: &str,
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

            match clone_repo(&created_repo, user, org, root, use_https) {
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

fn clone_repo(
    repo: &CreateRepo,
    user: &User,
    org: &str,
    root: &str,
    use_https: bool,
) -> Result<GitRepo> {
    let git_repo = repo.to_git_repo(org, user, root, use_https)?;

    let gp = git_repo.gclone()?;

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
        format!("{}.git", created_repo.html_url)
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
    fn to_git_repo(&self, org: &str, user: &User, root: &str, use_https: bool) -> Result<GitRepo> {
        let local_path = path::local_path_repo(org, &self.name, root);
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
