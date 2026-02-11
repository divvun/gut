use super::topic_add::*;
use super::topic_apply::*;
use super::topic_list::*;
use super::topic_set::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Add, list, set or apply a script by topic
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
    #[command(name = "list", alias = "get")]
    List(TopicListArgs),
    #[command(name = "set")]
    Set(TopicSetArgs),
}

impl TopicCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::List(args) => args.run(),
            Self::Set(args) => args.run(),
            Self::Add(args) => args.run(),
            Self::Apply(args) => args.run(),
        }
    }
}
