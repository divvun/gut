use crate::filter::Filter;
use anyhow::{anyhow, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub template: String,
    #[structopt(long, short)]
    pub version: Option<usize>,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        println!("Template apply args {:?}", self);
        Ok(())
    }
}
