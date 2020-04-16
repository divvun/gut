use super::common;
use crate::filter::Filter;
use crate::git;
use crate::path;
use crate::path::local_path_org;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
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
        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            if let Err(e) = commit(&dir, &self.message) {
                println!("Failed to commit dir {:?} because {:?}", dir, e);
            }
        }
        Ok(())
    }
}

fn commit(dir: &PathBuf, msg: &str) -> Result<()> {
    println!("{}", msg);
    Ok(())
}
