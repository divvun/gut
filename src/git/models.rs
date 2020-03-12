use crate::git::clone;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct GitRepo {
    pub remote_url: String,
    pub local_path: PathBuf,
}

impl clone::Clonable for GitRepo {
    type Output = GitRepo;

    fn gclone(&self) -> Result<Self::Output, clone::CloneError> {
        clone::clone(&self.remote_url, &self.local_path).map(|_| self.clone())
    }
}
