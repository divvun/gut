use anyhow::Result;
use git2::{Commit, Error, Index, Repository, Tree};

pub fn commit_tree(
    repo: &Repository,
    tree: &Tree,
    msg: &str,
    parents: &[&Commit],
) -> Result<(), Error> {
    let sig = repo.signature()?;
    let _merge_commit = repo.commit(Some("HEAD"), &sig, &sig, msg, tree, parents)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    Ok(())
}

pub fn commit_index(git_repo: &Repository, index: &mut Index, msg: &str) -> Result<(), Error> {
    let tree_id = index.write_tree()?;
    let result_tree = git_repo.find_tree(tree_id)?;

    let head_oid = git_repo.head()?.target().expect("Head needs oid");
    let head_commit = git_repo.find_commit(head_oid)?;

    commit_tree(&git_repo, &result_tree, msg, &[&head_commit])?;

    Ok(())
}

pub fn commit_first(git_repo: &Repository, index: &mut Index, msg: &str) -> Result<(), Error> {
    let tree_id = index.write_tree()?;
    let result_tree = git_repo.find_tree(tree_id)?;

    commit_tree(&git_repo, &result_tree, msg, &[])?;

    Ok(())
}
