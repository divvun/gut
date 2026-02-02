use super::path::config_path;
use super::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    pub root: String,
    /// Default owner (can be a GitHub organisation or user account)
    #[serde(alias = "default_org")]
    pub default_owner: Option<String>,
    pub use_https: bool,
}

impl Config {
    pub fn new(root: String, default_owner: Option<String>, use_https: bool) -> Config {
        Config {
            root,
            default_owner,
            use_https,
        }
    }

    pub fn save_config(&self) -> Result<()> {
        write_to_file(config_path()?, self)
    }

    pub fn from_file() -> Result<Config> {
        read_file(config_path()?)
    }

    pub fn root() -> Result<String> {
        Config::from_file().map(|c| c.root)
    }
}
