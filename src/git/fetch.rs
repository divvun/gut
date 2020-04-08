use super::common;
use super::models::GitCredential;
use git2::{Error, Repository};

// https://github.com/rust-lang/git2-rs/blob/master/examples/fetch.rs
pub fn fetch_branch(
    repo: &Repository,
    branch: &str,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    log::info!("Fetching {} for repo", branch);
    let mut remote = repo.find_remote(remote_name)?;

    let remote_callbacks = common::create_remote_callback(&cred)?;

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(remote_callbacks);

    remote.fetch(&[branch], Some(&mut fo), None)?;

    Ok(())
}
