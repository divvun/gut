use super::add_repos::*;
use super::add_users::*;
use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Add users, repos to an organisation/a team.
pub struct AddArgs {
    #[command(subcommand)]
    command: AddCommand,
}

impl AddArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
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
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            AddCommand::Users(args) => args.run(common_args),
            AddCommand::Repos(args) => args.run(common_args),
        }
    }
}
