use super::label_create::*;
use super::label_delete::*;
use super::label_list::*;
use super::label_rename::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// List, create, delete or rename labels
pub struct LabelArgs {
    #[command(subcommand)]
    command: LabelCommand,
}

impl LabelArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum LabelCommand {
    #[command(name = "list")]
    List(LabelListArgs),
    #[command(name = "create")]
    Create(LabelCreateArgs),
    #[command(name = "delete")]
    Delete(LabelDeleteArgs),
    #[command(name = "rename")]
    Rename(LabelRenameArgs),
}

impl LabelCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::List(args) => args.run(),
            Self::Create(args) => args.run(),
            Self::Delete(args) => args.run(),
            Self::Rename(args) => args.run(),
        }
    }
}
