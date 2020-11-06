use git2::{AnnotatedCommit, Error, Index, Repository};

#[derive(Debug)]
pub enum RebaseStatus {
    NormalRebase,
    RebaseWithConflict,
    SkipByConflict,
}


pub fn rebase_commit(
    repo: &Repository,
    annotated_commit: &AnnotatedCommit,
    abort_if_conflict: bool,
) -> Result<RebaseStatus, Error> {
    let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
    return normal_rebase(
        &repo,
        &head_commit,
        annotated_commit,
        abort_if_conflict,
    );
}

fn normal_rebase(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
    abort_if_conflict: bool,
) -> Result<RebaseStatus, git2::Error> {
    let mut operations = repo.rebase(Some(&local), Some(&remote), None, None)?;
    let sig = repo.signature()?;
    let mut result = RebaseStatus::NormalRebase;
    while let Some(operation) = operations.next() {
        let operation = operation?;
        match operation.kind() {
            Some(git2::RebaseOperationType::Exec) => {continue;}
            _ => {
                let idx = repo.index()?;
                if idx.has_conflicts() {
                    show_conflicts(&idx)?;
                    if abort_if_conflict {
                        operations.abort()?;
                        return Ok(RebaseStatus::SkipByConflict);
                    }
                    result = RebaseStatus::RebaseWithConflict;
                }
                operations.commit(None, &sig, None)?;
            }
        }
    }

    operations.finish(None)?;
    Ok(result)
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
