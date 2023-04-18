use std::convert::TryFrom;

use super::common;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::{Context, Result};
use clap::Parser;
use dryoc::dryocbox::{DryocBox, PublicKey};

#[derive(Debug, Parser)]
/// Set a secret all repositories that match regex
pub struct SecretArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
    #[arg(long, short, required_unless_present("website"))]
    /// The value for your secret
    pub value: String,
    #[arg(long, short, required_unless_present("description"))]
    /// The name of your secret
    pub name: String,
}

impl SecretArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, Some(&self.regex), &user_token)?;

        for repo in filtered_repos {
            let result = set_secret(&repo, &self.value, &self.name, &user_token);
            match result {
                Ok(_) => println!("Set secret value for repo {} successfully", repo.name),
                Err(e) => println!(
                    "Failed to set secret value for repo {} because {:?}",
                    repo.name, e
                ),
            }
        }
        Ok(())
    }
}

fn set_secret(repo: &RemoteRepo, value: &str, name: &str, token: &str) -> Result<()> {
    let public_key = github::get_public_key(repo, token)?;
    let encrypted_value = encrypt(value, &public_key.key)?;
    github::set_secret(repo, name, &encrypted_value, &public_key.key_id, token)?;
    Ok(())
}

fn encrypt(value: &str, key: &str) -> Result<String> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD;
    let bytes = b64.decode(key)?;
    let public_key = PublicKey::try_from(bytes.as_slice())
        .context("Invalide public key received from github")?;

    let encrypted = DryocBox::seal_to_vecbox(value.as_bytes(), &public_key)?;
    let encrypted = b64.encode(encrypted.to_vec());

    Ok(encrypted)
}
