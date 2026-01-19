use super::topic_add::*;
use super::topic_apply::*;
use super::topic_get::*;
use super::topic_set::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Add, get, set or apply a script by topic
pub struct TopicArgs {
    #[command(subcommand)]
    command: TopicCommand,
}

impl TopicArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
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
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Get(args) => args.run(),
            Self::Set(args) => args.run(),
            Self::Add(args) => args.run(),
            Self::Apply(args) => args.run(),
        }
    }
}
