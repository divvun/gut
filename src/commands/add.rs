use super::add_repos::*;
use super::add_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum AddArgs {
    #[structopt(name = "users")]
    Users(AddUsersArgs),
    #[structopt(name = "repos")]
    Repos(AddRepoArgs),
}

impl AddArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            AddArgs::Users(args) => args.run(),
            AddArgs::Repos(args) => args.run(),
        }
    }
}
