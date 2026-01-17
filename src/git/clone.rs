use super::common;
use super::models::GitCredential;
use std::io::{self, Write};
use std::path::Path;
use std::str;

pub trait Clonable {
    type Output;
    fn gclone(&self) -> Result<Self::Output, CloneError>;

    /*
    fn gclone_list<T: Clonable>(list: Vec<T>) -> Vec<Result<T::Output, CloneError>>
    where
        T: Send + Sync,
        T::Output: Send + Sync,
    {
        list.par_iter().map(|r| r.gclone()).collect()
    }
    */
}

#[derive(thiserror::Error, Debug)]
#[error("Cannot clone repository with {remote_url} because of {source}")]
pub struct CloneError {
    pub source: git2::Error,
    pub remote_url: String,
}

pub fn clone(
    remote_url: &str,
    local_path: &Path,
    cred: Option<GitCredential>,
    quiet: bool,
) -> Result<git2::Repository, CloneError> {
    if !quiet {
        log::debug!("Clone {:?} to {:?}", remote_url, local_path);
    }
    let mut callback = common::create_remote_callback(cred).map_err(|s| CloneError {
        source: s,
        remote_url: remote_url.to_string(),
    })?;

    // Set up progress callbacks that respect quiet mode
    callback.transfer_progress(move |stats| {
        if !quiet {
            if stats.received_objects() == stats.total_objects() {
                print!(
                    "Resolving deltas {}/{}\r",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                );
            } else if stats.total_objects() > 0 {
                print!(
                    "Received {}/{} objects ({}) in {} bytes\r",
                    stats.received_objects(),
                    stats.total_objects(),
                    stats.indexed_objects(),
                    stats.received_bytes()
                );
            }
            io::stdout().flush().unwrap();
        }
        true
    });

    callback.sideband_progress(move |data| {
        if !quiet {
            print!("remote: {}", str::from_utf8(data).unwrap());
            io::stdout().flush().unwrap();
        }
        true
    });

    let mut fo = git2::FetchOptions::new();

    fo.remote_callbacks(callback)
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

/*
#[cfg(test)]
mod tests {
    use super::super::models::{GitCredential, GitRepo};
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore]
    fn test_clone() -> anyhow::Result<()> {
        let user = crate::commands::common::user()?;
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
        let results: Vec<GitRepo> = results.unwrap().into_iter().collect();
        assert_eq!(vec, results);
        dir.close()?;
        Ok(())
    }
}
*/
