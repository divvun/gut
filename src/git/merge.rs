use git2::{Error, Repository};

// https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
pub fn merge_local(repo: &Repository, target: &str, base: &str, abort_if_conflict: bool) -> Result<(), Error> {
    log::info!("Merging {} into head", target);

    let refname = format!("refs/heads/{}", target);
    let target_ref = repo.find_reference(&refname)?;

    let annotated_commit = repo.reference_to_annotated_commit(&target_ref)?;

    let mut head_ref = repo.head()?;

    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&annotated_commit])?;

    if analysis.0.is_fast_forward() {
        log::debug!("fast forward");
        fast_forward(repo, &mut head_ref, &annotated_commit)?;
    } else if analysis.0.is_normal() {
        log::debug!("Normal merge");
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &annotated_commit, abort_if_conflict)?;
    }
    Ok(())
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    log::debug!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
    abort_if_conflict: bool,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        log::debug!("Merge conficts detected...");
        if abort_if_conflict {
            return Ok(());
        }
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }

    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    log::debug!("merge commit {:?}", _merge_commit);
    repo.checkout_tree(&result_tree.as_object(), None)?;
    repo.checkout_head(None)?;
    Ok(())
}
