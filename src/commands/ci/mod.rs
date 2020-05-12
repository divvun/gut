pub mod export;
pub mod generate;
pub mod models;

use anyhow::Result;
use export::*;
use generate::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum CiArgs {
    #[structopt(name = "export")]
    Export(ExportArgs),
    #[structopt(name = "generate")]
    Generate(GenerateArgs),
}

impl CiArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            CiArgs::Export(args) => args.run(),
            CiArgs::Generate(args) => args.run(),
        }
    }
}
