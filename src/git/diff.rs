use git2::{Error, Repository, Diff, DiffOptions};

pub fn diff_trees<'a>(repo: &'a Repository, old: &str, new: &str) -> Result<Diff<'a>, Error> {
    let old_tree = super::tree_from_commit_sha(repo, old)?;
    let new_tree = super::tree_from_commit_sha(repo, new)?;

    let mut opts = DiffOptions::new();

    repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut opts))
}
