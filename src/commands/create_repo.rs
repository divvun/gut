use super::common;
use crate::github::create_org_repo;
use std::path::PathBuf;

use crate::path::{local_path_org, Directory};
use anyhow::{anyhow, Context, Result};

use crate::filter::{Filter, Filterable};
//use crate::git::push;
use crate::git::open;
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
}

impl CreateRepoArgs {
    pub fn create_repo(&self) -> Result<()> {
        println!("Create Repo {:?}", self);
        let dir = match &self.dir {
            Some(d) => d.path.clone(),
            None => local_path_org(&self.organisation)?,
        };
        let sub_dirs = read_dirs(&dir, &self.regex)?;
        println!("Filtered sub dirs: {:?}", sub_dirs);
        let token = common::user_token()?;
        for dir in sub_dirs {
            match create_repo(&self.organisation, &dir, &self.public, &token) {
                Ok(_) => println!("Success"),
                Err(e) => println!("Failed because of {:?}", e),
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
        .into_iter()
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|x| x.is_dir())
        .collect();
    Ok(PathBuf::filter(dirs, filter))
}

/// Check if {dir} is a git repository
/// Create a new repository in organization {org}
/// Set the new created repository as remote origin
/// Push all to remote origin

fn create_repo(org: &str, dir: &PathBuf, public: &bool, token: &str) -> Result<()> {
    let local_repo =
        open::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let repo_name = dir
        .to_str()
        .ok_or(anyhow!("{:?} is not a valid name", dir))?;
    let new_repo = create_org_repo(org, repo_name, public, token)?;
    log::debug!("new created repo: {:?}", new_repo.html_url);
    Ok(())
}
