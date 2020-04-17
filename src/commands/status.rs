use super::common;
use crate::filter::Filter;
use crate::git;
use crate::path::local_path_org;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct StatusArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub verbose: bool,
}

impl StatusArgs {
    pub fn run(&self) -> Result<()> {
        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            if let Err(e) = status(&dir, self.verbose) {
                println!(
                    "Failed to status git status for dir {:?} because {:?}",
                    dir, e
                );
            }
        }
        Ok(())
    }
}

fn status(dir: &PathBuf, verbose: bool) -> Result<()> {
    println!("Status for {:?}:", dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let status = git::status(&git_repo)?;
    let current_branch = git::head_shorthand(&git_repo)?;

    println!("On branch {}", current_branch);

    if status.is_ahead > 0 {
        println!(
            "Your branch is ahead of 'origin/{}' by {} commits",
            current_branch, status.is_ahead
        );
    } else if status.is_behind > 0 {
        println!(
            "Your branch is behind of 'origin/{}' by {} commits",
            current_branch, status.is_ahead
        );
    } else {
        println!("Your branch is up to date with 'origin/{}'", current_branch);
    }

    println!();

    if status.is_empty() {
        println!("Nothing to commit, working tree is clean.");
    } else {
        println!("Changes:");

        if verbose {
            show_detail(&status);
        } else {
            show_summarize(&status);
        }
    }

    println!();

    Ok(())
}

fn show_summarize(status: &git::GitStatus) {
    show_number_of_changes("conflicted files:", &status.conflicted);
    show_number_of_changes("untracked files:", &status.new);
    show_number_of_changes("deleted files:", &status.deleted);
    show_number_of_changes("modified files:", &status.modified);
    show_number_of_changes("renamed files:", &status.renamed);
    show_number_of_changes("typechanges files:", &status.typechanges);
}

fn show_detail(status: &git::GitStatus) {
    show_detail_changes("conflicted files:", &status.conflicted);
    show_detail_changes("untracked files:", &status.new);
    show_detail_changes("deleted files:", &status.deleted);
    show_detail_changes("modified files:", &status.modified);
    show_detail_changes("renamed files:", &status.renamed);
    show_detail_changes("typechanges files:", &status.typechanges);
}

fn show_detail_changes(msg: &str, list: &[String]) {
    if !list.is_empty() {
        println!("{} {}", list.len(), msg);
        for l in list {
            println!("    {}", l);
        }
    }
}

fn show_number_of_changes(msg: &str, list: &[String]) {
    if !list.is_empty() {
        println!("{} {}", list.len(), msg);
    }
}
