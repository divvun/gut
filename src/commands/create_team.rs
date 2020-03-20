use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};
use std::convert::TryFrom;

use anyhow::{Context, Result};

use crate::filter::{Filter, Filterable};
use crate::user::User;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateTeamArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub team_name: String,
    #[structopt(long, short)]
    pub description: Option<String>,
    #[structopt(long, short)]
    pub secret: bool,
    #[structopt(long, short)]
    pub members: Vec<String>,
}

impl CreateTeamArgs {
    pub fn create_team(&self) -> Result<()> {
        Ok(())
    }
}
