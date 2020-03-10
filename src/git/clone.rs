use std::path::PathBuf;

use git2;
use git2_credentials::ui4dialoguer::CredentialUI4Dialoguer;
use git2_credentials::CredentialHandler;

use super::models::GitRepo;

pub trait Clonable {
    fn gclone(&self) -> Result<PathBuf, CloneError>;
}

#[derive(thiserror::Error, Debug)]
#[error("Cannot clone repository with {remote_url} because of this error {source}")]
pub struct CloneError {
    source: git2::Error,
    remote_url: String,
}

pub fn gclone<T>(repos: Vec<T>) -> Vec<Result<PathBuf, CloneError>>
where
    T: Clonable,
{
    repos.iter().map(|r| r.gclone()).collect()
}

impl Clonable for GitRepo {
    fn gclone(&self) -> Result<PathBuf, CloneError> {
        log::debug!("Do clone {:?}", self);
        let mut cb = git2::RemoteCallbacks::new();
        let git_config = git2::Config::open_default().map_err(|s| CloneError {
            source: s,
            remote_url: self.remote_url.clone(),
        })?;
        // Prepare callbacks.
        let mut ch =
            CredentialHandler::new_with_ui(git_config, Box::new(CredentialUI4Dialoguer {}));

        cb.credentials(move |url, username, allowed| {
            ch.try_next_credential(url, username, allowed)
        });

        let mut fo = git2::FetchOptions::new();

        fo.remote_callbacks(cb)
            .download_tags(git2::AutotagOption::All)
            .update_fetchhead(true);

        git2::build::RepoBuilder::new()
            .fetch_options(fo)
            .clone(&self.remote_url, self.local_path.as_ref())
            .map_err(|s| CloneError {
                source: s,
                remote_url: self.remote_url.clone(),
            })?;
        Ok(self.local_path.clone())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore]
    fn test_clone() -> anyhow::Result<()> {
        let dir = tempdir()?;

        let repo1_path = dir.path().join("rxarrow");
        let repo1 = GitRepo {
            remote_url: "https://github.com/lenguyenthanh/rxarrow".to_string(),
            local_path: repo1_path.clone(),
        };

        let repo2_path = dir.path().join("beckon");
        let repo2 = GitRepo {
            remote_url: "git@gitlab.com:technocreatives/beckon/beckon-android.git".to_string(),
            local_path: repo2_path.clone(),
        };

        let repo3_path = dir.path().join("dadmin");
        let repo3 = GitRepo {
            remote_url: "git@gitlab.com:technocreatives/divvun-uit/dadmin.git".to_string(),
            local_path: repo3_path.clone(),
        };

        let repo4_path = dir.path().join("nimble");
        let repo4 = GitRepo {
            remote_url: "git@github.com:lenguyenthanh/nimble.git".to_string(),
            local_path: repo4_path.clone(),
        };

        let vec = vec![repo1, repo2, repo3, repo4];

        let results = dbg!(gclone(vec));
        let paths: Result<Vec<PathBuf>, CloneError> = results.into_iter().collect();
        let paths = paths.unwrap();

        let expected_results = vec![repo1_path, repo2_path, repo3_path, repo4_path];

        assert_eq!(expected_results, paths);
        dir.close()?;
        Ok(())
    }
}
