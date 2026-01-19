use super::show_config::*;
use super::show_repos::*;
use super::show_users::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Show config, list of repositories or users
pub struct ShowArgs {
    #[command(subcommand)]
    command: ShowCommand,
}

impl ShowArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
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
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Config => show_config(),
            Self::Repos(args) => args.show(),
            Self::Users(args) => args.run(),
        }
    }
}
