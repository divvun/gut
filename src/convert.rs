use crate::git::models::{GitCredential, GitRepo};
use crate::github::RemoteRepo;
use crate::path::local_path;
use crate::user::User;
use std::io::{Error, ErrorKind};

pub fn try_from_one(repo: RemoteRepo, user: &User, use_https: bool) -> Result<GitRepo, Error> {
    let local_path = local_path(&repo.owner, &repo.name)
        .ok_or_else(|| Error::new(ErrorKind::Other, "Cannot create local path"))?;

    let remote_url = if use_https {
        format!("{}.git", repo.https_url)
    } else {
        repo.ssh_url
    };

    let cred = GitCredential::from(user);

    Ok(GitRepo {
        remote_url,
        local_path,
        cred: Some(cred),
    })
}

pub fn try_from(vec: Vec<RemoteRepo>, user: &User, use_https: bool) -> Result<Vec<GitRepo>, Error> {
    vec.into_iter()
        .map(|repo| try_from_one(repo, user, use_https))
        .collect()
}
