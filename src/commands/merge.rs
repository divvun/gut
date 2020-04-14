use super::common;
use super::models::Script;
use crate::filter::Filter;
use crate::path::local_path_org;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::{Command, Output};
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
    pub abort_if_confict: bool,
}

impl MergeArgs {
    pub fn run(&self) -> Result<()> {
        log::debug!("Merge run {:?}", self);

        Ok(())
    }
}
