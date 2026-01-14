use super::set_default_organisation::*;
use super::set_info::*;
use super::set_secret::*;
use super::set_team_permission::*;
use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Set information, secret for repositories or permission for a team
pub struct SetArgs {
    #[command(subcommand)]
    command: SetCommand,
}

impl SetArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
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
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Info(args) => args.run(common_args),
            Self::Organisation(args) => args.run(common_args),
            Self::Permission(args) => args.set_permission(common_args),
            Self::Secret(args) => args.run(common_args),
        }
    }
}
