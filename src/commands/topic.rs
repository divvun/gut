use super::topic_set::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TopicArgs {
    #[structopt(name = "set")]
    Set(TopicSetArgs),
    #[structopt(name = "update")]
    Update(TopicSetArgs),
}

impl TopicArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            TopicArgs::Set(args) => args.run(),
            TopicArgs::Update(args) => args.run(),
        }
    }
}
