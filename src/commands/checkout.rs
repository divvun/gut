use super::common;
use crate::git;
use crate::path::local_path_org;
use crate::user::User;

use anyhow::{Context, Result};

use crate::filter::Filter;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CheckoutArgs {
    #[structopt(long, short)]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Filter,
    #[structopt(long, short)]
    pub branch: String,
}

impl CheckoutArgs {
    pub fn run(&self) -> Result<()> {
        log::debug!("checkout branch {:?}", self);
        let user = common::user()?;

        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            match checkout_branch(&dir, &self.branch, &user, &"origin") {
                Ok(_) => println!(
                    "Checkout branch {} of repo {:?} successfully",
                    &self.branch, dir
                ),
                Err(e) => println!(
                    "Failed to checkout branch {} of repo {:?} because {:?}",
                    &self.branch, dir, e
                ),
            }
        }

        Ok(())
    }
}

fn checkout_branch(dir: &PathBuf, branch: &str, user: &User, remote_name: &str) -> Result<()> {
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    git::checkout_local_branch(&git_repo, branch)?;

    Ok(())
}
