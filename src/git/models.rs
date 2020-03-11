use super::clone::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct GitRepo {
    pub remote_url: String,
    pub local_path: PathBuf,
}

impl Clonable for GitRepo {
    type Output = PathBuf;

    fn gclone(&self) -> Result<Self::Output, CloneError> {
        clone(&self.remote_url, &self.local_path)
    }
}
