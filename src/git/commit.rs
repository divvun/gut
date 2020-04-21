use git2::{Commit, Error, Repository, Tree};

pub fn commit(repo: &Repository, tree: &Tree, msg: &str, parents: &[&Commit]) -> Result<(), Error> {
    let sig = repo.signature()?;
    let _merge_commit = repo.commit(Some("HEAD"), &sig, &sig, msg, tree, parents)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    Ok(())
}
