use super::add_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum AddArgs {
    #[structopt(name = "users")]
    Users(AddUsersArgs),
}

impl AddArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            AddArgs::Users(args) => args.add_users(),
        }
    }
}
