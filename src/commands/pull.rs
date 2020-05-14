use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::git::MergeStatus;
use crate::path;
use crate::user::User;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Pull the current branch of all local repositories that match a regex
///
/// This command only works on those repositories that has been cloned in root directory
pub struct PullArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    //#[structopt(long, short)]
    //pub abort_if_conflict: bool,
}

impl PullArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        for dir in sub_dirs {
            let dir_name = path::dir_name(&dir)?;
            println!("Pulling for {}", dir_name);

            match pull(&dir, &user, false) {
                Ok(status) => match status {
                    MergeStatus::FastForward => println!("Pull success by merge fast forward"),
                    MergeStatus::NormalMerge => {
                        println!("Pull success by merge with 'recursive' strategy")
                    }
                    MergeStatus::MergeWithConflict => {
                        println!("Auto merge failed. Fix conflicts and then commit the results.")
                    }
                    MergeStatus::Nothing => println!("Already up to date"),
                    MergeStatus::SkipByConflict => {
                        println!("There are conflict(s), and we skipped")
                    }
                },
                Err(e) => println!("Failed to pull for dir {:?} because {:?}", dir, e),
            }
            println!("===============");
        }

        Ok(())
    }
}

fn pull(dir: &PathBuf, user: &User, abort_if_conflict: bool) -> Result<MergeStatus> {
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let cred = GitCredential::from(user);
    let status = git::pull(&git_repo, "origin", Some(cred), abort_if_conflict)?;

    Ok(status)
}
