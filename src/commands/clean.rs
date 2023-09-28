use super::common;
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Do git clean -f for all local repositories that match a pattern
pub struct CleanArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
}

impl CleanArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let root = common::root()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let sub_dirs = common::read_dirs_for_org(&organisation, &root, self.regex.as_ref())?;

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
