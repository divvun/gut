use super::path::config_path;
use super::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    root: String,
}

impl Config {
    pub fn new(root: String) -> Config {
        Config { root }
    }

    pub fn save_config(&self) -> Result<()> {
        write_to_file(path(), self)
    }

    pub fn config() -> Result<Config> {
        read_file(path())
    }

    pub fn root() -> Result<String> {
        Config::config().map(|c| c.root)
    }
}

fn path() -> PathBuf {
    let path = config_path();
    log::info!("Conifg path: {:?}", path);
    match path {
        Some(p) => p,
        None => panic!("Cannot read the config directory. We need to read our config file in your config directory."),
    }
}
