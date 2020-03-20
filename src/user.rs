use super::github;
use super::path::user_path;
use super::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    token: String,
}

impl User {
    pub fn new(token: String) -> Result<User> {
        github::is_valid_token(&token)?;
        let user = User { token };
        println!("Authorization successful!");
        Ok(user)
    }

    pub fn save_user(&self) -> Result<()> {
        write_to_file(get_path(), self)
    }

    pub fn get_user() -> Result<User> {
        read_file(get_path())
    }

    pub fn get_token() -> Result<String> {
        let user = User::get_user()?;
        Ok(user.token)
    }
}

fn get_path() -> PathBuf {
    let path = user_path();
    log::info!("User path: {:?}", path);
    match path {
        Some(p) => p,
        None => panic!("Cannot read the config directory. We need to read our config file in your config directory."),
    }
}
