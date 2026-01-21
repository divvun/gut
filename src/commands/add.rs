use super::add_repos::*;
use super::add_users::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Add users (to an organisation or team) or repositories (to a team)
pub struct AddArgs {
    #[command(subcommand)]
    command: AddCommand,
}

impl AddArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum AddCommand {
    #[command(name = "users")]
    Users(AddUsersArgs),
    #[command(name = "repos")]
    Repos(AddRepoArgs),
}

impl AddCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            AddCommand::Users(args) => args.run(),
            AddCommand::Repos(args) => args.run(),
        }
    }
}
