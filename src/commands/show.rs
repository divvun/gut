use super::show_config::*;
use super::show_repos::*;
use super::show_users::*;
use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Show config, list of repositories or users
pub struct ShowArgs {
    #[command(subcommand)]
    command: ShowCommand,
}

impl ShowArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum ShowCommand {
    /// Show current configuration
    #[command(name = "config")]
    Config,
    #[command(name = "repositories", aliases = &["repos"])]
    Repos(ShowReposArgs),
    #[command(name = "users")]
    Users(ShowUsersArgs),
}

impl ShowCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Config => show_config(common_args),
            Self::Repos(args) => args.show(common_args),
            Self::Users(args) => args.run(common_args),
        }
    }
}
