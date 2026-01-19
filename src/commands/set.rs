use super::set_default_organisation::*;
use super::set_info::*;
use super::set_secret::*;
use super::set_team_permission::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Set information, secret for repositories or permission for a team
pub struct SetArgs {
    #[command(subcommand)]
    command: SetCommand,
}

impl SetArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum SetCommand {
    #[command(name = "info")]
    Info(InfoArgs),
    #[command(name = "organisation")]
    Organisation(SetOrganisationArgs),
    #[command(name = "permission")]
    Permission(SetTeamPermissionArgs),
    #[command(name = "secret")]
    Secret(SecretArgs),
}

impl SetCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Info(args) => args.run(),
            Self::Organisation(args) => args.run(),
            Self::Permission(args) => args.set_permission(),
            Self::Secret(args) => args.run(),
        }
    }
}
