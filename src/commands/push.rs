use anyhow::Result;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PushArgs {
    #[structopt(long, short)]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub branch: String,
}

impl PushArgs {
    pub fn run(&self) -> Result<()> {
        println!("push branch {:?}", self);
        Ok(())
    }
}
