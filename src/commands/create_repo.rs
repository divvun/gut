use std::path::PathBuf;
use std::fs::DirEntry;
use super::common;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;

use crate::path::{Directory, local_path_org};
use anyhow::{Result, anyhow};

use crate::filter::{Filter, Filterable};
use crate::git::branch;
use crate::git::push;
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
        let sub_dirs = read_dirs(&dir, &self.regex);
        println!("Filtered sub dirs: {:?}", sub_dirs);
        Ok(())
    }
}

fn read_dirs(path: &PathBuf, filter: &Filter) -> Result<Vec<PathBuf>> {
    let entries = path.read_dir()?;
    let dirs = entries.into_iter()
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|x| x.is_dir())
        .collect();
    Ok(PathBuf::filter(dirs, filter))
}
