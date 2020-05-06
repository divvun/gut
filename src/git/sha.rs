use anyhow::Result;
use git2::Repository;

pub fn head_sha(repo: &Repository) -> Result<String> {
    let head_oid = repo.head()?.target().expect("Head needs oid");
    Ok(head_oid.to_string())
}
