use super::branch_default::*;
use super::branch_protect::*;
use super::branch_unprotect::*;
use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Set default, set protected branch
pub struct BranchArgs {
    #[command(subcommand)]
    command: BranchCommand,
}

impl BranchArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum BranchCommand {
    #[command(name = "default")]
    Default(DefaultBranchArgs),
    #[command(name = "protect")]
    Protect(ProtectedBranchArgs),
    #[command(name = "unprotect")]
    Unprotect(UnprotectedBranchArgs),
}

impl BranchCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            BranchCommand::Default(args) => args.set_default_branch(common_args),
            BranchCommand::Protect(args) => args.set_protected_branch(common_args),
            BranchCommand::Unprotect(args) => args.set_unprotected_branch(common_args),
        }
    }
}
