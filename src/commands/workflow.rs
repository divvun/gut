use super::workflow_run::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Run a workflow
pub enum WorkflowArgs {
    #[structopt(name = "run")]
    Run(WorkflowRunArgs),
}

impl WorkflowArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            WorkflowArgs::Run(args) => args.run(),
        }
    }
}
