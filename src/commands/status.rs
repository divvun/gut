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
        println!("Run");
        Ok(())
    }
}
