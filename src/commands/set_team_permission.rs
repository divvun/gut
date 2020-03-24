use super::common;
use crate::github::RemoteRepo;
use std::convert::TryFrom;

use anyhow::Result;

use crate::filter::Filter;
use crate::git::branch;
use crate::git::models::GitRepo;
use crate::git::push;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SetTeamPermissionArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub team_slug: String,
    #[structopt(long, short)]
    pub permission: String,
}

impl SetTeamPermissionArgs {
    pub fn set_permission(&self) -> Result<()> {
        println!("SetTeamPermissionArgs {:?}", self);
        Ok(())
    }
}
