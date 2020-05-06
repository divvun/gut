use super::apply::*;
use super::generate::*;
use anyhow::Result;
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
