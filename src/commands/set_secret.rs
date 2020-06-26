use super::common;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::{Context, Result};
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::PublicKey;
use sodiumoxide::crypto::sealedbox;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set a secret all repositories that match regex
pub struct SecretArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
    #[structopt(long, short, required_unless("website"))]
    /// The value for your secret
    pub value: String,
    #[structopt(long, short, required_unless("description"))]
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
    let bytes = base64::decode(key)?;
    let public_key = PublicKey::from_slice(&bytes).context("Invalid public key from github")?;
    let encrypted = sealedbox::seal(value.as_bytes(), &public_key);
    let encrypted = base64::encode(encrypted);
    Ok(encrypted)
}
