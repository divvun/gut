use super::github;
use super::path::user_path;
use super::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub token: String,
    pub username: String,
}

impl User {
    pub fn new(token: String) -> Result<User> {
        let username = github::is_valid_token(&token)?;
        let user = User { token, username };
        println!("Authorization successful!");
        Ok(user)
    }

    pub fn save_user(&self) -> Result<()> {
        write_to_file(
            path().ok_or_else(|| anyhow::anyhow!("No user path found"))?,
            self,
        )
    }

    pub fn user() -> Result<User> {
        read_file(path().ok_or_else(|| anyhow::anyhow!("No user path found"))?)
    }

    pub fn token() -> Result<String> {
        let user = User::user()?;
        Ok(user.token)
    }
}

fn path() -> Option<PathBuf> {
    let path = user_path();
    log::info!("User path: {:?}", path);
    path
}
