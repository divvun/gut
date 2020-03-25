use crate::git::models::GitRepo;
use crate::github::RemoteRepo;
use crate::path::get_local_path;
use std::io::{Error, ErrorKind};

pub fn try_from_one(repo: RemoteRepo, use_https: bool) -> Result<GitRepo, Error> {
    let local_path = get_local_path(&repo.owner, &repo.name).ok_or_else(|| {
        Error::new(ErrorKind::Other, "Cannot create local path")
    })?;

    let remote_url = if use_https {
        format!("{}.git", repo.https_url)
    } else {
        repo.ssh_url
    };
    Ok(GitRepo {
        remote_url,
        local_path,
    })
}

pub fn try_from(vec: Vec<RemoteRepo>, use_https: bool) -> Result<Vec<GitRepo>, Error> {
    vec.into_iter().map(|repo| try_from_one(repo, use_https)).collect()
}
