use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::toml::{read_file, write_to_file};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    root: String,
    name: String,
    email: String,
}

impl Config {
    pub fn new(root: String, name: String, email: String) -> Config {
        Config { root, name, email }
    }

    pub fn save_config(&self) -> Result<()> {
        write_to_file("config.toml", self)
    }

    pub fn get_config() -> Result<Config> {
        read_file("config.toml")
    }
}
