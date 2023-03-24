use super::path::config_path;
use super::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    pub root: String,
    pub default_org: Option<String>,
    pub use_https: bool,
}

impl Config {
    pub fn new(root: String, default_org: Option<String>, use_https: bool) -> Config {
        Config { root, default_org, use_https }
    }

    pub fn save_config(&self) -> Result<()> {
        write_to_file(path(), self)
    }

    pub fn from_file() -> Result<Config> {
        read_file(path())
    }

    pub fn root() -> Result<String> {
        Config::from_file().map(|c| c.root)
    }
}

fn path() -> PathBuf {
    let path = config_path();
    match path {
        Some(p) => p,
        None => panic!("Cannot read the config directory. We need to read our config file in your config directory."),
    }
}
