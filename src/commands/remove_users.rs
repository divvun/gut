use super::common;
use crate::github;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RemoveUsersArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub users: Vec<String>,
    #[structopt(long, short)]
    pub team_slug: Option<String>,
}

impl RemoveUsersArgs {
    pub fn remove_users(&self) -> Result<()> {
        match &self.team_slug {
            Some(name) => self.remove_users_for_team(&name),
            None => self.remove_users_for_org(),
        }
    }

    fn remove_users_for_team(name: &str) -> Result<()>{
        Ok(())
    }

    fn remove_users_for_org() -> Result<()> {
        Ok(())
    }
}
