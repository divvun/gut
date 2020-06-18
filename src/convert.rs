use crate::config::Config;
use crate::git::models::{GitCredential, GitRepo};
use crate::github::RemoteRepo;
use crate::path::local_path_repo;
use crate::user::User;
use anyhow::{Context, Result};

pub fn try_from_one(repo: RemoteRepo, user: &User, use_https: bool) -> Result<GitRepo> {
    let root = Config::root().context(
        "Cannot read the config file. Run `gut init` with a valid Github token and a root directory path",
    )?;

    let local_path = local_path_repo(&repo.owner, &repo.name, &root);

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

//pub fn try_from(vec: Vec<RemoteRepo>, user: &User, use_https: bool) -> Result<Vec<GitRepo>> {
//vec.into_par_iter()
//.map(|repo| try_from_one(repo, user, use_https))
//.collect()
//}
