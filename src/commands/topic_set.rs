use super::common;
use crate::filter::Filter;
use crate::path;
use anyhow::{Context, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct TopicSetArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub topics: Vec<String>,
}

impl TopicSetArgs {
    pub fn run(&self) -> Result<()> {
        println!("topic set {:?}", self);
        Ok(())
    }
}
