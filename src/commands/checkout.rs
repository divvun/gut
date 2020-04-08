use super::common;
use crate::git;
use crate::git::GitCredential;
use crate::path::local_path_org;
use crate::user::User;

use anyhow::{anyhow, Context, Result};

use crate::filter::Filter;
use git2::BranchType;
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
    #[structopt(long)]
    pub remote: bool,
}

impl CheckoutArgs {
    pub fn run(&self) -> Result<()> {
        log::debug!("checkout branch {:?}", self);
        let user = common::user()?;

        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            match checkout_branch(&dir, &self.branch, &user, &"origin", self.remote) {
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

fn checkout_branch(
    dir: &PathBuf,
    branch: &str,
    user: &User,
    remote_name: &str,
    remote: bool,
) -> Result<()> {
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    if git_repo.find_branch(branch, BranchType::Local).is_ok() {
        git::checkout_local_branch(&git_repo, branch)?;
    } else if remote {
        let cred = GitCredential::from(user);
        git::checkout_remote_branch(&git_repo, branch, remote_name, Some(cred))?;
    } else {
        return Err(anyhow!("There is no local branch with name: {}.\n You can use `--remote` option to checkout a remote branch.", branch));
    };

    Ok(())
}
