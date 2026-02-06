use super::rename_repos::*;
use super::rename_team::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Rename repositories or teams
pub struct RenameArgs {
    #[command(subcommand)]
    command: RenameCommand,
}

impl RenameArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum RenameCommand {
    #[command(name = "repos")]
    Repos(RenameReposArgs),
    #[command(name = "team")]
    Team(RenameTeamArgs),
}

impl RenameCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Repos(args) => args.run(),
            Self::Team(args) => args.run(),
        }
    }
}
