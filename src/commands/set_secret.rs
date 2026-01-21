use std::convert::TryFrom;

use super::common;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::{Context, Result};
use clap::Parser;
use dryoc::dryocbox::{DryocBox, PublicKey};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Set a secret for all repositories that match a regex
pub struct SecretArgs {
    #[arg(long, short, alias = "organisation")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
    #[arg(long, short)]
    /// The value for your secret
    pub value: String,
    #[arg(long, short)]
    /// The name of your secret
    pub name: String,
}

impl SecretArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let owner = common::owner(self.owner.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&owner, Some(&self.regex), &user_token)?;

        filtered_repos.par_iter().for_each(|repo| {
            let result = set_secret(repo, &self.value, &self.name, &user_token);
            match result {
                Ok(_) => println!("Set secret value for repo {} successfully", repo.name),
                Err(e) => println!(
                    "Failed to set secret value for repo {} because {:?}",
                    repo.name, e
                ),
            }
        });
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
    let public_key =
        PublicKey::try_from(bytes.as_slice()).context("Invalid public key received from github")?;

    let encrypted = DryocBox::seal_to_vecbox(value.as_bytes(), &public_key)?;
    let encrypted = b64.encode(encrypted.to_vec());

    Ok(encrypted)
}
