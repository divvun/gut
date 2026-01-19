use super::workflow_run::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Run a workflow
pub struct WorkflowArgs {
    #[command(subcommand)]
    command: WorkflowCommand,
}

impl WorkflowArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum WorkflowCommand {
    #[command(name = "run")]
    Run(WorkflowRunArgs),
}

impl WorkflowCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Run(args) => args.run(),
        }
    }
}
