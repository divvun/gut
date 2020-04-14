use super::common;
use crate::filter::Filter;
use crate::git;
use crate::path::local_path_org;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct MergeArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub target_branch: String,
    #[structopt(long, short, default_value = "master")]
    pub base_branch: String,
    #[structopt(long, short)]
    pub abort_if_conflict: bool,
}

impl MergeArgs {
    pub fn run(&self) -> Result<()> {
        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            match merge(
                &dir,
                &self.target_branch,
                &self.base_branch,
                self.abort_if_conflict,
            ) {
                Ok(_) => println!(
                    "Merged branch {} into {} for {:?} successfully",
                    self.target_branch, self.base_branch, dir
                ),
                Err(e) => println!(
                    "Failed to merge branch {} into branch {} for dir {:?} because {:?}",
                    self.target_branch, self.base_branch, dir, e
                ),
            }
        }

        Ok(())
    }
}

fn merge(dir: &PathBuf, target: &str, base: &str, abort: bool) -> Result<()> {
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    git::merge_local(&git_repo, target, base, abort)?;
    Ok(())
}
