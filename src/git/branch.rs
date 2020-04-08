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

pub fn checkout_local_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let obj = repo.revparse_single(&("refs/heads/".to_owned() + branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&("refs/heads/".to_owned() + branch_name))?;

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

    match repo.find_branch(&remote_branch, BranchType::Remote) {
        Err(_) => Err(anyhow!("There is no remote branch named: {}", branch)),
        Ok(found_branch) => {
            let oid = found_branch.get().target().unwrap();
            let commit = repo.find_commit(oid)?;
            repo.branch(&branch, &commit, false)?;
            checkout_local_branch(repo, branch)
        }
    }
}
