use super::github;
use super::path::user_path;
use super::toml::{read_file, write_to_file};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const KEYRING_SERVICE: &str = "gut";

fn keyring_entry(username: &str) -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, username).context("Failed to create keyring entry")
}

/// Serialised form of the user file. The `token` field exists only for
/// reading legacy files; it is never written.
#[derive(Serialize, Deserialize, Debug)]
struct UserFile {
    username: String,
    #[serde(default, skip_serializing)]
    token: Option<String>,
}

#[derive(Debug, Clone)]
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
        let username = self.username.clone();
        let token = self.token.clone();
        std::thread::spawn(move || -> Result<()> {
            keyring_entry(&username)?
                .set_password(&token)
                .context("Failed to save token to keyring")
        })
        .join()
        .map_err(|_| anyhow!("keyring thread panicked"))??;
        let file = UserFile {
            username: self.username.clone(),
            token: None,
        };
        write_to_file(user_path()?, &file)
    }

    pub fn from_config() -> Result<User> {
        let file: UserFile = read_file(user_path()?)?;

        // Migrate legacy files that still contain a plaintext token.
        if let Some(ref legacy_token) = file.token {
            let username = file.username.clone();
            let token = legacy_token.clone();
            std::thread::spawn(move || -> Result<()> {
                keyring_entry(&username)?
                    .set_password(&token)
                    .context("Failed to migrate token to keyring")
            })
            .join()
            .map_err(|_| anyhow!("keyring thread panicked"))??;
            let clean = UserFile {
                username: file.username.clone(),
                token: None,
            };
            write_to_file(user_path()?, &clean)?;
        }

        let username = file.username.clone();
        let token = std::thread::spawn(move || {
            keyring_entry(&username).ok()?.get_password().ok()
        })
        .join()
        .ok()
        .flatten()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .context(
            "No GitHub token found. Run `gut init --token <PAT>` or set the GITHUB_TOKEN \
             environment variable.",
        )?;

        Ok(User {
            token,
            username: file.username,
        })
    }

    pub fn token() -> Result<String> {
        let user = User::from_config()?;
        Ok(user.token)
    }
}
