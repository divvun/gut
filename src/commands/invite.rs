use super::invite_users::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Invite users to an owner by emails
pub struct InviteArgs {
    #[command(subcommand)]
    command: InviteCommand,
}

impl InviteArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum InviteCommand {
    #[command(name = "users")]
    Users(InviteUsersArgs),
}

impl InviteCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Users(args) => args.run(),
        }
    }
}
