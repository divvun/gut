use super::common;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;

use anyhow::Result;

use crate::filter::Filter;
use structopt::StructOpt;
use crate::git::push;
use git2::Repository;

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
        log::debug!("push branch {:?}", self);

        let user = common::user()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, &self.regex, &user.token)?;

        Ok(())
    }
}

fn push_branch(
    repo: &Repository,
    branch: &str,
    user: &User,
) -> Result<()> {

    Ok(())
}
