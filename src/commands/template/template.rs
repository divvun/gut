use super::apply::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TemplateArgs {
    #[structopt(name = "apply")]
    Apply(ApplyArgs),
}

impl TemplateArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            TemplateArgs::Apply(args) => args.run(),
        }
    }
}

