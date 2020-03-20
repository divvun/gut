use crate::git::models::GitRepo;
use crate::github::RemoteRepo;
use crate::path::get_local_path;
use std::convert::TryFrom;

impl TryFrom<RemoteRepo> for GitRepo {
    type Error = std::io::Error;

    fn try_from(repo: RemoteRepo) -> Result<Self, Self::Error> {
        let local_path = get_local_path(&repo.owner, &repo.name).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Cannot create local path")
        })?;
        Ok(GitRepo {
            remote_url: repo.ssh_url,
            local_path,
        })
    }
}

pub fn try_from<T, U: TryFrom<T>>(vec: impl IntoIterator<Item = T>) -> Result<Vec<U>, U::Error> {
    vec.into_iter().map(U::try_from).collect()
}
