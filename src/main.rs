mod cli;
mod config;
mod toml;

use anyhow::Result;
use cli::{Args, ConfigArgs};
use structopt::StructOpt;

use config::Config;

fn main() -> Result<()> {
    color_backtrace::install();

    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("dadmin"), log::LevelFilter::Debug)
        .init();

    let args = Args::from_args();
    log::debug!("{:?}", args);

    match args {
        Args::Init(ConfigArgs { root, name, email }) => {
            let config = Config::new(root, name, email);
            config.save_config()
        }
        _ => Ok(()),
    }
}
