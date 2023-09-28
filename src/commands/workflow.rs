use super::workflow_run::*;
use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    command: WorkflowCommand,
}
/// Run a workflow
impl WorkflowArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum WorkflowCommand {
    #[command(name = "run")]
    Run(WorkflowRunArgs),
}

impl WorkflowCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Run(args) => args.run(common_args),
        }
    }
}
