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
}

impl StatusArgs {
    pub fn run(&self) -> Result<()> {
        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            if let Err(e) = status(&dir) {
                println!(
                    "Failed to status git status for dir {:?} because {:?}",
                    dir, e
                );
            }
        }
        Ok(())
    }
}

fn status(dir: &PathBuf) -> Result<()> {
    println!("Status for {:?}:", dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let status = git::status(&git_repo)?;

    if status.is_empty() {
        println!("Nothing change!")
    } else {
        show_status("New files:", &status.new);
        show_status("Deleted files:", &status.deleted);
        show_status("Modified files:", &status.modified);
        show_status("Renamed files:", &status.renamed);
        show_status("Typechanges files:", &status.typechanges);
    }

    println!();

    Ok(())
}

fn show_status(msg: &str, list: &[String]) {
    if !list.is_empty() {
        println!("{}", msg);
        for l in list {
            println!("{}", l);
        }
    }
}
