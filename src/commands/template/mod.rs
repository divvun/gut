pub mod apply;
pub mod generate;
pub mod patch_file;
pub mod refresh;

use anyhow::Result;
use apply::*;
use generate::*;
use refresh::*;

use clap::Parser;

#[derive(Debug, Parser)]
/// Apply changes or generate new template
pub struct TemplateArgs {
    #[command(subcommand)]
    command: TemplateCommand,
}

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
    #[command(name = "refresh")]
    Refresh(RefreshArgs),
}

impl TemplateCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Apply(args) => args.run(),
            Self::Generate(args) => args.run(),
            Self::Refresh(args) => args.run(),
        }
    }
}
