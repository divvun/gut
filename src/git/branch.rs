use super::fetch;
use super::models::GitCredential;
use anyhow::{anyhow, Result};
use git2::{Branch, BranchType, Error, Repository};

pub trait CreateBranch<'a> {
    fn create_branch(&self, new_branch: &str, base_branch: &str) -> Result<Branch<'a>, Error>;
}

pub fn create_branch<'a>(
    repo: &'a Repository,
    new_branch: &str,
    base_branch: &str,
) -> Result<Branch<'a>, Error> {
    let base_branch = repo.find_branch(base_branch, BranchType::Local)?;

    // unwrap work here because I assume branch always has direct reference
    let oid = base_branch.get().target().unwrap();
    let commit = repo.find_commit(oid)?;
    repo.branch(new_branch, &commit, false)
}

pub fn checkout_local_branch<'a>(repo: &'a Repository, branch: &str) -> Result<()> {
    let ref_to_branch = format!("refs/heads/{}", branch);
    repo.set_head(&ref_to_branch)?;
    Ok(())
}

pub fn checkout_remote_branch<'a>(
    repo: &'a Repository,
    branch: &str,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<()> {
    log::debug!("checkout remote branch");

    if let Err(e) = fetch::fetch_branch(repo, branch, remote_name, cred) {
        return Err(anyhow!("Cannot fetch branch {} because {:?}", branch, e));
    }

    let remote_branch = format!("{}/{}", remote_name, branch);

    if repo
        .find_branch(&remote_branch, BranchType::Remote)
        .is_err()
    {
        return Err(anyhow!("There is no remote branch named: {}", branch));
    } else {
        checkout_local_branch(repo, branch)?;
    }

    Ok(())
}
