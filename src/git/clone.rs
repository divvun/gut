use super::models::GitCredential;
use git2_credentials::ui4dialoguer::CredentialUI4Dialoguer;
use std::path::Path;

use git2;
use git2_credentials::CredentialHandler;
use git2_credentials::CredentialUI;

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

pub fn clone(
    remote_url: &str,
    local_path: &Path,
    cred: Option<GitCredential>,
) -> Result<git2::Repository, CloneError> {
    log::debug!("Clone {:?} to {:?}", remote_url, local_path);
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default().map_err(|s| CloneError {
        source: s,
        remote_url: remote_url.to_string(),
    })?;

    let credential_ui: Box<dyn CredentialUI> = match cred {
        Some(gc) => Box::new(gc),
        _ => Box::new(CredentialUI4Dialoguer {}),
    };

    // Prepare callbacks.
    let mut ch = CredentialHandler::new_with_ui(git_config, credential_ui);

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
        })
}

#[cfg(test)]
mod tests {
    use super::super::models::{GitCredential, GitRepo};
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore]
    fn test_clone() -> anyhow::Result<()> {
        let user = crate::commands::common::get_user()?;
        let cred = Some(GitCredential::from(&user));

        let dir = tempdir()?;
        let repo1_path = dir.path().join("public-ssh-1");
        let repo1 = GitRepo {
            remote_url: "git@github.com:dadmin-test/test-1.git".to_string(),
            local_path: repo1_path,
            cred: cred.clone(),
        };
        let repo2_path = dir.path().join("public-https-1");
        let repo2 = GitRepo {
            remote_url: "https://github.com/dadmin-test/test-1.git".to_string(),
            local_path: repo2_path,
            cred: cred.clone(),
        };
        let repo3_path = dir.path().join("private-https-1");
        let repo3 = GitRepo {
            remote_url: "https://github.com/dadmin-test/private-test-1.git".to_string(),
            local_path: repo3_path,
            cred: cred.clone(),
        };
        let repo4_path = dir.path().join("private-ssh-1");
        let repo4 = GitRepo {
            remote_url: "git@github.com:dadmin-test/private-test-1.git".to_string(),
            local_path: repo4_path,
            cred,
        };

        let vec = vec![repo1, repo2, repo3, repo4];
        let results = GitRepo::gclone_list(vec.clone());
        let results: Result<Vec<_>, CloneError> = results.into_iter().collect();
        let results: Vec<GitRepo> = results.unwrap().into_iter().map(|(g, _)| g).collect();
        assert_eq!(vec, results);
        dir.close()?;
        Ok(())
    }
}
