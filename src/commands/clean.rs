use super::common;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CleanArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl CleanArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, &self.regex.as_ref())?;

        for dir in sub_dirs {
            if let Err(e) = clean(&dir) {
                println!("Failed to clean dir {:?} because {:?}", dir, e);
            }
        }
        Ok(())
    }
}

fn clean(dir: &PathBuf) -> Result<()> {
    println!("Cleaning {:?}", dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let status = git::status(&git_repo, false)?;
    //println!("git status {:?}", status);

    if status.new.is_empty() {
        println!("Nothing to clean!\n");
    } else {
        println!("Files/directories get removed: ");
        for f in status.new {
            let rf = dir.join(f);
            path::remove_path(&rf).with_context(|| format!("Cannot remove {:?}", rf))?;
            println!("{:?}", rf);
        }
        println!();
    }

    Ok(())
}
