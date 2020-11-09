use super::branch;
use super::fetch;
use super::merge;
use super::models::GitCredential;
use super::rebase;
use anyhow::Result;
use git2::Repository;
use std::str;

#[derive(Debug)]
pub enum PullStatus {
    Normal,
    Nothing,
    FastForward,
    SkipConflict,
    WithConflict,
}

pub fn pull(
    repo: &Repository,
    remote_name: &str,
    cred: Option<GitCredential>,
    merge: bool,
) -> Result<PullStatus> {
    let branch_name = branch::head_shorthand(repo)?;
    let fetch_commit = fetch::fetch_branch(repo, &branch_name, remote_name, cred)?;

    if merge {
        let msg = format!(
            "Merge branch \'{}\' of {} into {}",
            branch_name, remote_name, branch_name
        );
        let status = merge::merge_commit(repo, &fetch_commit, &msg, false)?;
        Ok(status.into())
    } else {
        let status = rebase::rebase_commit(repo, &fetch_commit, false)?;
        Ok(status.into())
    }
}

impl From<merge::MergeStatus> for PullStatus {
    fn from(status: merge::MergeStatus) -> Self {
        match status {
            super::MergeStatus::FastForward => PullStatus::FastForward,
            super::MergeStatus::NormalMerge => PullStatus::Normal,
            super::MergeStatus::MergeWithConflict => PullStatus::WithConflict,
            super::MergeStatus::SkipByConflict => PullStatus::SkipConflict,
            super::MergeStatus::Nothing => PullStatus::Nothing,
        }
    }
}

impl From<rebase::RebaseStatus> for PullStatus {
    fn from(status: rebase::RebaseStatus) -> Self {
        match status {
            super::RebaseStatus::NormalRebase => PullStatus::Normal,
            super::RebaseStatus::RebaseWithConflict => PullStatus::WithConflict,
            super::RebaseStatus::SkipByConflict => PullStatus::SkipConflict,
        }
    }
}
