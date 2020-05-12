pub mod apply;
pub mod generate;
pub mod patch_file;

use anyhow::Result;
use apply::*;
use generate::*;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TemplateArgs {
    #[structopt(name = "apply")]
    Apply(ApplyArgs),
    #[structopt(name = "generate")]
    Generate(GenerateArgs),
}

impl TemplateArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            TemplateArgs::Apply(args) => args.run(),
            TemplateArgs::Generate(args) => args.run(),
        }
    }
}
