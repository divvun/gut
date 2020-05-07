use anyhow::Result;
use git2::{Commit, Oid, Repository};

pub fn head_sha(repo: &Repository) -> Result<String> {
    let head_oid = repo.head()?.target().expect("Head needs oid");
    Ok(head_oid.to_string())
}

pub fn get_commit<'a>(repo: &'a Repository, sha: &str) -> Result<Commit<'a>> {
    let oid = Oid::from_str(sha)?;
    let commit = repo.find_commit(oid)?;
    Ok(commit)
}
