use super::common;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;

use anyhow::Result;

use crate::filter::Filter;
use crate::git::push;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct PushBranchArgs {
    #[structopt(long, short)]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub branch: String,
}

impl PushBranchArgs {
    pub fn push_branch(&self) -> Result<()> {
        println!("push branch {:?}", self);
        Ok(())
    }
}
