pub mod apply;
pub mod generate;
pub mod patch_file;
pub mod refresh;

use crate::cli::Args as CommonArgs;
use anyhow::Result;
use apply::*;
use generate::*;
use refresh::*;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[command(subcommand)]
    command: TemplateCommand,
}
/// Apply changes or generate new template
impl TemplateArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
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
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Apply(args) => args.run(common_args),
            Self::Generate(args) => args.run(common_args),
            Self::Refresh(args) => args.run(common_args),
        }
    }
}
