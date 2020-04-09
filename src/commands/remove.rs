use super::remove_repo::*;
use super::remove_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum RemoveArgs {
    #[structopt(name = "users")]
    Users(RemoveUsersArgs),
    #[structopt(name = "repo")]
    Repos(RemoveReposArgs),
}

impl RemoveArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            RemoveArgs::Users(args) => args.run(),
            RemoveArgs::Repos(args) => args.run(),
        }
    }
}
