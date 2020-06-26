use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::MergeStatus;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Merge a branch to the current branch for all repositories that match a pattern
pub struct MergeArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// The branch to be merged
    pub branch: String,
    #[structopt(long, short)]
    /// Option to abort merging process if there is a conflict
    pub abort_if_conflict: bool,
}

impl MergeArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let sub_dirs = common::read_dirs_for_org(&organisation, &root, self.regex.as_ref())?;

        for dir in sub_dirs {
            match merge(&dir, &self.branch, self.abort_if_conflict) {
                Ok(status) => match status {
                    MergeStatus::FastForward => println!("Merge fast forward"),
                    MergeStatus::NormalMerge => println!("Merge made by the 'recursive' strategy"),
                    MergeStatus::MergeWithConflict => {
                        println!("Auto merge failed. Fix conflicts and then commit the results.")
                    }
                    MergeStatus::Nothing => println!("Already up to date"),
                    MergeStatus::SkipByConflict => {
                        println!("There are conflict(s), and we skipped")
                    }
                },
                Err(e) => println!(
                    "Failed to merge branch {} for dir {:?} because {:?}",
                    self.branch, dir, e
                ),
            }
        }

        Ok(())
    }
}

fn merge(dir: &PathBuf, target: &str, abort: bool) -> Result<git::MergeStatus> {
    println!("Merging branch {} into head for {:?}", target, dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let merge_status = git::merge_local(&git_repo, target, abort)?;
    Ok(merge_status)
}
