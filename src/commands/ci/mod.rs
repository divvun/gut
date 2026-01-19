pub mod export;
pub mod generate;
pub mod models;

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
    pub fn run(&self) -> Result<()> {
        self.command.run()
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
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Export(args) => args.run(),
            Self::Generate(args) => args.run(),
        }
    }
}
