use super::show_config::*;
use super::show_repos::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum ShowArgs {
    #[structopt(name = "config")]
    Config,
    #[structopt(name = "repositories", aliases = &["repos"])]
    Repos(ShowReposArgs),
}

impl ShowArgs {
    pub fn show(&self) -> Result<()> {
        match self {
            ShowArgs::Config => show_config(),
            ShowArgs::Repos(args) => args.show(),
        }
    }
}
