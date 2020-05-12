use super::hook_create::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum HookArgs {
    #[structopt(name = "create")]
    Create(CreateArgs),
}

impl HookArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            HookArgs::Create(args) => args.run(),
        }
    }
}
