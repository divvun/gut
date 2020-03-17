use crate::git::clone;
use git2::{Error, Repository};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct GitRepo {
    pub remote_url: String,
    pub local_path: PathBuf,
}

impl clone::Clonable for GitRepo {
    type Output = (GitRepo, git2::Repository);

    fn gclone(&self) -> Result<Self::Output, clone::CloneError> {
        clone::clone(&self.remote_url, &self.local_path).map(|r| (self.clone(), r))
    }
}

impl GitRepo {
    pub fn open(&self) -> Result<Repository, Error> {
        match Repository::open(&self.local_path) {
            Ok(repo) => Ok(repo),
            Err(_) => clone::clone(&self.remote_url, &self.local_path).map_err(|e| e.source),
        }
    }
}
