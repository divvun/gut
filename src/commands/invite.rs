use crate::cli::Args as CommonArgs;
use super::invite_users::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct InviteArgs {
    #[command(subcommand)]
    command: InviteCommand,
}
/// Invite users to an organisation by emails
impl InviteArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum InviteCommand {
    #[command(name = "users")]
    Users(InviteUsersArgs),
}

impl InviteCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Users(args) => args.run(common_args),
        }
    }
}
