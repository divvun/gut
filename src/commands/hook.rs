use crate::cli::Args as CommonArgs;
use super::hook_create::*;
use super::hook_delete::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct HookArgs {
    #[command(subcommand)]
    command: HookCommand,
}
/// Create, delete hooks for all repositories that match a pattern
impl HookArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum HookCommand {
    #[command(name = "create")]
    Create(CreateArgs),
    #[command(name = "delete")]
    Delete(DeleteArgs),
}

impl HookCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Create(args) => args.run(common_args),
            Self::Delete(args) => args.run(common_args),
        }
    }
}
