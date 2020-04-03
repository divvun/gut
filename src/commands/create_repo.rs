use super::common;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;

use crate::path::Directory;
use anyhow::Result;

use crate::filter::Filter;
use crate::git::branch;
use crate::git::push;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateRepoArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Filter,
    #[structopt(long, short)]
    pub dir: Option<Directory>,
    #[structopt(long, short)]
    pub public: bool,
}

impl CreateRepoArgs {
    pub fn create_repo(&self) -> Result<()> {
        println!("Create Repo {:?}", self);
        Ok(())
    }
}
