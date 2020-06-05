use anyhow::Result;
use git2::{Oid, Repository};

pub fn stash(repo: &mut Repository, msg: Option<&str>) -> Result<Oid> {
    let sig = repo.signature()?;
    let oid = repo.stash_save2(&sig, msg, None)?;
    Ok(oid)
}

//pub fn apply(repo: &mut Repository) -> Result<()> {
//repo.stash_apply(0, None)?;
//Ok(())
//}
