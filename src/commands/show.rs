use super::show_access::*;
use super::show_config::*;
use super::show_members::*;
use super::show_repos::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Show config, repositories, members, or user access
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
    #[command(name = "repositories", visible_aliases = &["repos"])]
    Repos(ShowReposArgs),
    #[command(name = "access")]
    Access(ShowAccessArgs),
    #[command(name = "members", visible_aliases = &["users"])]
    Members(ShowMembersArgs),
}

impl ShowCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Config => show_config(),
            Self::Repos(args) => args.show(),
            Self::Access(args) => args.run(),
            Self::Members(args) => args.run(),
        }
    }
}
