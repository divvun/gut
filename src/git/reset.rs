use anyhow::Result;
use git2::{Repository, ResetType};

pub fn reset_hard(repo: &Repository, commit_oid: git2::Oid) -> Result<()> {
    let commit = repo.find_commit(commit_oid)?;
    let object = commit.as_object();
    repo.reset(object, ResetType::Hard, None)?;
    Ok(())
}

pub fn reset_hard_head(repo: &Repository) -> Result<()> {
    let head = repo.head()?;
    let commit_oid = head
        .target()
        .ok_or_else(|| anyhow::anyhow!("HEAD has no target"))?;
    reset_hard(repo, commit_oid)
}
