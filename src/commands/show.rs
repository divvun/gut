use super::show_config::*;
use super::show_repos::*;
use super::show_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum ShowArgs {
    #[structopt(name = "config")]
    Config,
    #[structopt(name = "repositories", aliases = &["repos"])]
    Repos(ShowReposArgs),
    #[structopt(name = "users")]
    Users(ShowUsersArgs),
}

impl ShowArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            ShowArgs::Config => show_config(),
            ShowArgs::Repos(args) => args.show(),
            ShowArgs::Users(args) => args.run(),
        }
    }
}
