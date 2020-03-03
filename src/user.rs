use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::toml::{read_file, write_to_file};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    token: String,
}

impl User {
    pub fn new(token: String) -> Result<User> {
        super::api::is_valid_token(&token)?;
        let user = User { token };
        println!("Authorization successful!");
        Ok(user)
    }

    pub fn save_user(&self) -> Result<()> {
        write_to_file("user.toml", self)
    }

    pub fn get_user() -> Result<User> {
        read_file("user.toml")
    }
}
