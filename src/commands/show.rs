use super::show_config::*;
use super::show_repos::*;
use super::show_users::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Show config, list of repositories or users
pub enum ShowArgs {
    #[structopt(name = "config")]
    // Show current configuration
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
