use git2::{Error, Oid, Repository, Tree};

pub fn tree_from_commit_sha<'a>(repo: &'a Repository, sha: &str) -> Result<Tree<'a>, Error> {
    // println!("Get tree from {:?} with sha {}", repo.path(), sha);
    let oid = Oid::from_str(sha)?;
    let commit = repo.find_commit(oid)?;
    commit.tree()
}
