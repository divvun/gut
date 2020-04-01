use super::remove_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum RemoveArgs {
    #[structopt(name = "users")]
    Users(RemoveUsersArgs),
}

impl RemoveArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            RemoveArgs::Users(args) => args.remove_users(),
        }
    }
}
