use super::hook_create::*;
use super::hook_delete::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum HookArgs {
    #[structopt(name = "create")]
    Create(CreateArgs),
    #[structopt(name = "delete")]
    Delete(DeleteArgs),
}

impl HookArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            HookArgs::Create(args) => args.run(),
            HookArgs::Delete(args) => args.run(),
        }
    }
}
