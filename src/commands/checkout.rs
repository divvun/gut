use super::common;
use crate::git::open;
use crate::path::local_path_org;
use crate::user::User;

use anyhow::{Context, Result};

use crate::filter::Filter;
use crate::git::push;
use crate::git::GitCredential;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CheckoutArgs {
    #[structopt(long, short)]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Filter,
    #[structopt(long, short)]
    pub branch: String,
}

impl CheckoutArgs {
    pub fn run(&self) -> Result<()> {
        log::debug!("checkout branch {:?}", self);
        Ok(())
    }
}
