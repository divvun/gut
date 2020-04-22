use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::path;
use crate::user::User;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct FetchArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl FetchArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let target_dir = path::local_path_org(&self.organisation)?;
        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            fetch(&dir, &user)?;
        }
        Ok(())
    }
}

fn fetch(dir: &PathBuf, user: &User) -> Result<()> {
    let dir_name = path::dir_name(dir)?;
    println!("Fetching for {}", dir_name);

    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let cred = GitCredential::from(user);
    git::fetch(&git_repo, "origin", Some(cred))?;

    println!("===============");
    Ok(())
}
