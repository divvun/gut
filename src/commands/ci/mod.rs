pub mod export;
pub mod generate;
pub mod models;

use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;
use export::*;
use generate::*;

#[derive(Debug, Parser)]
/// Generate or export ci configuration
pub struct CiArgs {
    #[command(subcommand)]
    command: CiCommand,
}

impl CiArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum CiCommand {
    #[command(name = "export")]
    Export(ExportArgs),
    #[command(name = "generate")]
    Generate(GenerateArgs),
}

impl CiCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Export(args) => args.run(common_args),
            Self::Generate(args) => args.run(common_args),
        }
    }
}
