use super::common;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Add all and then commit with the provided messages
pub struct CommitArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub message: String,
}

impl CommitArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        for dir in sub_dirs {
            let dir_name = path::dir_name(&dir)?;
            match commit(&dir, &self.message) {
                Err(e) => println!("{}: Failed to commit because {:?}", dir_name, e),
                Ok(result) => match result {
                    CommitResult::Conflict => println!(
                        "{}: There are conflicts. Fix conflicts and then commit the results.",
                        dir_name
                    ),
                    CommitResult::NoChanges => println!("{}: There is no changes.", dir_name),
                    CommitResult::Success => println!("{}: Commit success.", dir_name),
                },
            }
        }
        Ok(())
    }
}

fn commit(dir: &PathBuf, msg: &str) -> Result<CommitResult> {
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let status = git::status(&git_repo, true)?;
    //let current_branch = git::head_shorthand(&git_repo)?;

    if !status.can_commit() {
        return Ok(CommitResult::Conflict);
    }

    if !status.should_commit() {
        return Ok(CommitResult::NoChanges);
    }

    let mut index = git_repo.index()?;

    let addable_list = status.addable_list();
    for p in addable_list {
        //log::debug!("addable file: {}", p);
        let path = Path::new(&p);
        index.add_path(path)?;
    }

    for p in status.deleted {
        //log::debug!("removed file: {}", p);
        let path = Path::new(&p);
        index.remove_path(path)?;
    }

    let tree_id = index.write_tree()?;
    let result_tree = git_repo.find_tree(tree_id)?;

    let head_oid = git_repo.head()?.target().expect("Head needs oid");
    let head_commit = git_repo.find_commit(head_oid)?;

    git::commit(&git_repo, &result_tree, msg, &[&head_commit])?;

    Ok(CommitResult::Success)
}

enum CommitResult {
    Conflict,
    NoChanges,
    Success,
}
