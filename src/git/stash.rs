use anyhow::Result;
use git2::{Repository, Oid};

pub fn stash(repo: &mut Repository, msg: Option<&str>) -> Result<Oid> {
    let sig = repo.signature()?;
    let oid = repo.stash_save2(&sig, msg, None)?;
    Ok(oid)
}
