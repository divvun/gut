pub mod apply;
pub mod generate;
pub mod patch_file;

use anyhow::Result;
use apply::*;
use generate::*;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[command(subcommand)]
    command: TemplateCommand,
}
/// Apply changes or generate new template
impl TemplateArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum TemplateCommand {
    #[command(name = "apply")]
    Apply(ApplyArgs),
    #[command(name = "generate")]
    Generate(GenerateArgs),
}

impl TemplateCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Apply(args) => args.run(),
            Self::Generate(args) => args.run(),
        }
    }
}
