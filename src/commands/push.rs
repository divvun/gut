use super::common;
use crate::git::open;
use crate::user::User;

use anyhow::{Context, Result};

use crate::filter::Filter;
use crate::git::push;
use crate::git::GitCredential;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PushArgs {
    #[structopt(long, short)]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Filter,
    #[structopt(long, short)]
    pub branch: String,
}

impl PushArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, Some(&self.regex))?;

        for dir in sub_dirs {
            match push_branch(&dir, &self.branch, &user, &"origin") {
                Ok(_) => println!(
                    "Pushed branch {} of repo {:?} successfully",
                    &self.branch, dir
                ),
                Err(e) => println!(
                    "Failed to push branch {} of repo {:?} because {:?}",
                    &self.branch, dir, e
                ),
            }
        }

        Ok(())
    }
}

fn push_branch(dir: &PathBuf, branch: &str, user: &User, remote_name: &str) -> Result<()> {
    let git_repo = open::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let cred = GitCredential::from(user);
    push::push_branch(&git_repo, branch, remote_name, Some(cred))?;
    Ok(())
}
