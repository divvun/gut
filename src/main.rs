mod cli;
mod commands;
mod config;
mod convert;
mod filter;
mod git;
mod github;
mod path;
mod toml;
mod user;

use anyhow::Result;
use cli::{Args, Commands, InitArgs};
use config::Config;
use structopt::StructOpt;
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
                Err(e) => match e.downcast_ref::<github::Unauthorized>() {
                    Some(_) => anyhow::bail!("Token is invalid. Check https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"),
                    _ => return Err(e)
                }
            };
            user.save_user()?;
            let config = Config::new(root.path);
            config.save_config()
        }
        Commands::ListRepos(list_repo_args) => list_repo_args.show(),
        Commands::CloneRepos(clone_args) => clone_args.clone(),
        Commands::CreateBranch(args) => args.create_branch(),
        Commands::DefaultBranch(args) => args.set_default_branch(),
        Commands::ProtectedBranch(args) => args.set_protected_branch(),
        Commands::CreateTeam(args) => args.create_team(),
        Commands::AddUsers(args) => args.add_users(),
        _ => Ok(()),
    }
}
