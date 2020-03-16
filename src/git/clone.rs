use std::path::{Path, PathBuf};

use git2;
use git2_credentials::ui4dialoguer::CredentialUI4Dialoguer;
use git2_credentials::CredentialHandler;

pub trait Clonable {
    type Output;
    fn gclone(&self) -> Result<Self::Output, CloneError>;
    fn gclone_list<T: Clonable>(list: Vec<T>) -> Vec<Result<T::Output, CloneError>> {
        list.iter().map(|r| r.gclone()).collect()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Cannot clone repository with {remote_url} because of this error {source}")]
pub struct CloneError {
    pub source: git2::Error,
    pub remote_url: String,
}

pub fn clone(remote_url: &str, local_path: &Path) -> Result<PathBuf, CloneError> {
    log::debug!("Clone {:?} to {:?}", remote_url, local_path);
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default().map_err(|s| CloneError {
        source: s,
        remote_url: remote_url.to_string(),
    })?;
    // Prepare callbacks.
    let mut ch = CredentialHandler::new_with_ui(git_config, Box::new(CredentialUI4Dialoguer {}));

    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    let mut fo = git2::FetchOptions::new();

    fo.remote_callbacks(cb)
        .download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);

    git2::build::RepoBuilder::new()
        .fetch_options(fo)
        .clone(remote_url, local_path)
        .map_err(|s| CloneError {
            source: s,
            remote_url: remote_url.to_string(),
        })?;
    Ok(local_path.to_path_buf())
}
