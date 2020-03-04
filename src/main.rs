mod api;
mod cli;
mod config;
mod path;
mod toml;
mod user;

use anyhow::Result;
use cli::{Args, InitArgs};
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

    match args {
        Args::Init(InitArgs { root, token }) => {
            let user = match User::new(token) {
                Ok(user) => { user },
                Err(e) => match e.downcast_ref::<api::Error>() {
                    Some(api::Error::Unauthorized) => anyhow::bail!("Token is invalid. Check https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"),
                    _ => return Err(e)
                }
            };
            user.save_user()?;
            let config = Config::new(root);
            config.save_config()
        }
        _ => Ok(()),
    }
}
