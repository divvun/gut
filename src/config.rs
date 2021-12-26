use super::path::config_path;
use super::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub root: String,
    pub default_org: Option<String>,
}

impl Config {
    pub fn new(root: String, default_org: Option<String>) -> Config {
        Config { root, default_org }
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
    match path {
        Ok(p) => p,
        Err(e) => panic!("{}\n Cannot read the config directory. We need to read our config file in your config directory.", e),
    }
}
