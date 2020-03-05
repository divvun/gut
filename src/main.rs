mod api;
mod cli;
mod config;
mod path;
mod toml;
mod user;

use crate::api::list_repos;
use anyhow::{Context, Result};
use cli::{Args, Commands, InitArgs, ListRepoArgs};
use structopt::StructOpt;

use config::Config;
use user::User;

fn main() -> Result<()> {
    color_backtrace::install();

    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("dadmin"), log::LevelFilter::Debug)
        .init();

    let args = Args::from_args();
    log::debug!("Arguments: {:?}", args);

    match args.command {
        Commands::Init(InitArgs { root, token }) => {
            let user = match User::new(token) {
                Ok(user) => { user },
                Err(e) => match e.downcast_ref::<api::Unauthorized>() {
                    Some(_) => anyhow::bail!("Token is invalid. Check https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"),
                    _ => return Err(e)
                }
            };
            user.save_user()?;
            let config = Config::new(root);
            config.save_config()
        }
        Commands::ListRepos(ListRepoArgs { organisation }) => {
            let repos = match list_repos(&organisation).context("Fetching repositories") {
                Ok(repos) => repos,
                Err(e) => {
                    if let Some(_) = e.downcast_ref::<api::NoReposFound>() {
                        anyhow::bail!("No repositories found");
                    }
                    if let Some(_) = e.downcast_ref::<api::Unauthorized>() {
                        anyhow::bail!("User token invalid. Run dadmin init with a valid token");
                    }
                    return Err(e);
                }
            };

            for repo in &repos {
                println!("{}", repo);
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
