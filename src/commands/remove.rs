use super::remove_repos::*;
use super::remove_users::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct RemoveArgs {
    #[command(subcommand)]
    command: RemoveCommand,
}
/// Remove users, repos from an organisation/a team.
impl RemoveArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum RemoveCommand {
    #[command(name = "users")]
    Users(RemoveUsersArgs),
    #[command(name = "repositories", aliases = &["repos"])]
    Repos(RemoveReposArgs),
}

impl RemoveCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Users(args) => args.run(),
            Self::Repos(args) => args.run(),
        }
    }
}
