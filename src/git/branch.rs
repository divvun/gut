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