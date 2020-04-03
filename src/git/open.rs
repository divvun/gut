use super::clone;
use super::models::GitCredential;
use git2::{Error, Repository};
use std::path::PathBuf;

pub fn open(path: &PathBuf) -> Result<Repository, Error> {
    Repository::open(path)
}

pub fn open_or_clone(
    local_path: &PathBuf,
    remote_url: &str,
    cred: &Option<GitCredential>,
) -> Result<Repository, Error> {
    match Repository::open(local_path) {
        Ok(repo) => Ok(repo),
        Err(_) => clone::clone(remote_url, local_path, cred.clone()).map_err(|e| e.source),
    }
}
