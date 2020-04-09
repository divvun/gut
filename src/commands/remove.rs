use super::remove_repos::*;
use super::remove_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum RemoveArgs {
    #[structopt(name = "users")]
    Users(RemoveUsersArgs),
    #[structopt(name = "repositories", aliases = &["repos"])]
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
