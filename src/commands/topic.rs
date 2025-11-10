use super::topic_add::*;
use super::topic_apply::*;
use super::topic_get::*;
use super::topic_set::*;
use crate::cli::Args as CommonArgs;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct TopicArgs {
    #[command(subcommand)]
    command: TopicCommand,
}
/// Add, get, set or apply a script by topic
impl TopicArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
    }
}

#[derive(Debug, Parser)]
pub enum TopicCommand {
    #[command(name = "add")]
    Add(TopicAddArgs),
    #[command(name = "apply")]
    Apply(TopicApplyArgs),
    #[command(name = "get")]
    Get(TopicGetArgs),
    #[command(name = "set")]
    Set(TopicSetArgs),
}

impl TopicCommand {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Get(args) => args.run(common_args),
            Self::Set(args) => args.run(common_args),
            Self::Add(args) => args.run(common_args),
            Self::Apply(args) => args.run(common_args),
        }
    }
}
