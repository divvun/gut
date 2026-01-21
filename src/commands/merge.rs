use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::git;
use crate::git::MergeStatus;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Merge a branch into the current branch for all repositories that match a pattern
pub struct MergeArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// The branch to be merged
    pub branch: String,
    #[arg(long, short = 'x')]
    /// Abort merging process if there is a conflict
    pub abort_if_conflict: bool,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

impl MergeArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Merged",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_owner(owner, &root, self.regex.as_ref())?;

        let total_count = sub_dirs.len();
        let mut success_count = 0;
        let mut fail_count = 0;

        for dir in sub_dirs {
            match merge(&dir, &self.branch, self.abort_if_conflict) {
                Ok(status) => {
                    success_count += 1;
                    match status {
                        MergeStatus::FastForward => println!("Merge fast forward"),
                        MergeStatus::NormalMerge => {
                            println!("Merge made by the 'recursive' strategy")
                        }
                        MergeStatus::MergeWithConflict => {
                            println!(
                                "Auto merge failed. Fix conflicts and then commit the results."
                            )
                        }
                        MergeStatus::Nothing => println!("Already up to date"),
                        MergeStatus::SkipByConflict => {
                            println!("There are conflict(s), and we skipped")
                        }
                    }
                }
                Err(e) => {
                    fail_count += 1;
                    println!(
                        "Failed to merge branch {} for dir {:?} because {:?}",
                        self.branch, dir, e
                    );
                }
            }
        }

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: total_count,
            successful_repos: success_count,
            failed_repos: fail_count,
            dirty_repos: 0,
        })
    }
}

fn merge(dir: &PathBuf, target: &str, abort: bool) -> Result<git::MergeStatus> {
    println!("Merging branch {} into head for {:?}", target, dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let merge_status = git::merge_local(&git_repo, target, abort)?;
    Ok(merge_status)
}
