use super::common;
use super::models::Script;
use crate::filter::Filter;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub script: Script,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        let script_path = self
            .script
            .path
            .to_str()
            .expect("dadmin only supports utf8 path now!");

        for dir in sub_dirs {
            match common::apply_script(&dir, script_path) {
                Ok(_) => println!(
                    "Applied script {} for dir {:?} successfully",
                    script_path, dir
                ),
                Err(e) => println!(
                    "Failed to apply script {} for dir {:?} because {:?}",
                    script_path, dir, e
                ),
            }
        }

        Ok(())
    }
}

