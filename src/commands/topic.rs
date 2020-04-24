use super::topic_add::*;
use super::topic_get::*;
use super::topic_set::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Sub command for set/get/add topics
pub enum TopicArgs {
    #[structopt(name = "add")]
    Add(TopicAddArgs),
    #[structopt(name = "get")]
    Get(TopicGetArgs),
    #[structopt(name = "set")]
    Set(TopicSetArgs),
}

impl TopicArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            TopicArgs::Get(args) => args.run(),
            TopicArgs::Set(args) => args.run(),
            TopicArgs::Add(args) => args.run(),
        }
    }
}
