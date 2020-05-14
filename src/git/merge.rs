use super::commit;
use git2::{AnnotatedCommit, Error, Index, Repository};

pub enum MergeStatus {
    FastForward,
    NormalMerge,
    MergeWithConflict,
    SkipByConflict,
    Nothing,
}

// https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs
pub fn merge_local(
    repo: &Repository,
    target: &str,
    abort_if_conflict: bool,
) -> Result<MergeStatus, Error> {
    let refname = format!("refs/heads/{}", target);
    let target_ref = repo.find_reference(&refname)?;
    let annotated_commit = repo.reference_to_annotated_commit(&target_ref)?;
    let msg = format!("Merge branch '{}'", target);
    merge_commit(repo, &annotated_commit, &msg, abort_if_conflict)
}

pub fn merge_commit(
    repo: &Repository,
    annotated_commit: &AnnotatedCommit,
    msg: &str,
    abort_if_conflict: bool,
) -> Result<MergeStatus, Error> {
    let mut head_ref = repo.head()?;

    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[annotated_commit])?;

    if analysis.0.is_fast_forward() {
        return fast_forward(repo, &mut head_ref, annotated_commit);
    } else if analysis.0.is_normal() {
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        return normal_merge(
            &repo,
            &head_commit,
            annotated_commit,
            msg,
            abort_if_conflict,
        );
    }
    Ok(MergeStatus::Nothing)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<MergeStatus, git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: {} to id: {}", name, rc.id());
    //log::debug!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(MergeStatus::FastForward)
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
    msg: &str,
    abort_if_conflict: bool,
) -> Result<MergeStatus, git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        //log::debug!("Merge conficts detected...");
        show_conflicts(&idx)?;
        if abort_if_conflict {
            return Ok(MergeStatus::SkipByConflict);
        }

        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(MergeStatus::MergeWithConflict);
    }

    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    commit::commit_tree(repo, &result_tree, &msg, &[&local_commit, &remote_commit])?;
    Ok(MergeStatus::NormalMerge)
}

fn show_conflicts(idx: &Index) -> Result<(), Error> {
    let conflitcs = idx.conflicts()?;
    for c in conflitcs {
        if let Some(id) = c?.our {
            println!(
                "CONFLICT (content): Merge conflict in {:?}",
                String::from_utf8_lossy(&id.path)
            );
        }
    }
    Ok(())
}
