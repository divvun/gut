use super::common;
use crate::filter::Filter;
use crate::git;
use crate::path::local_path_org;
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
        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            match clean(&dir) {
                Ok(list) => {
                    for l in list {
                        println!("Removing {}", l);
                    }
                }
                Err(e) => println!("Failed to clean because {:?}", e),
            }
        }
        Ok(())
    }
}

fn clean(dir: &PathBuf) -> Result<Vec<String>> {
    println!("Cleaning dir {:?}", dir);
    Ok(Vec::new())
}
