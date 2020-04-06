use super::open;
use crate::git::clone;
use crate::user::User;
use dialoguer::PasswordInput;
use git2::{Error, Repository};
use git2_credentials::CredentialUI;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct GitRepo {
    pub remote_url: String,
    pub local_path: PathBuf,
    pub cred: Option<GitCredential>,
}

impl clone::Clonable for GitRepo {
    type Output = (GitRepo, git2::Repository);

    fn gclone(&self) -> Result<Self::Output, clone::CloneError> {
        clone::clone(&self.remote_url, &self.local_path, self.cred.clone())
            .map(|r| (self.clone(), r))
    }
}

impl GitRepo {
    pub fn open_or_clone(&self) -> Result<Repository, Error> {
        open::open_or_clone(&self.local_path, &self.remote_url, &self.cred)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GitCredential {
    username: String,
    password: String,
}

impl GitCredential {
    pub fn new(username: String, password: String) -> GitCredential {
        GitCredential { username, password }
    }
}

impl CredentialUI for GitCredential {
    fn ask_user_password(&self, _: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        Ok((self.username.clone(), self.password.clone()))
    }

    fn ask_ssh_passphrase(
        &self,
        passphrase_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let passphrase: String = PasswordInput::new()
            .with_prompt(passphrase_prompt)
            .allow_empty_password(true)
            .interact()?;
        Ok(passphrase)
    }
}

impl From<&User> for GitCredential {
    fn from(user: &User) -> GitCredential {
        GitCredential::new(user.username.clone(), user.token.clone())
    }
}
