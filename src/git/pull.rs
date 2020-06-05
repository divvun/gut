use super::branch;
use super::fetch;
use super::merge;
use super::models::GitCredential;
use anyhow::Result;
use git2::Repository;
use std::str;

pub fn pull(
    repo: &Repository,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<merge::MergeStatus> {
    let branch_name = branch::head_shorthand(repo)?;
    let fetch_commit = fetch::fetch_branch(repo, &branch_name, remote_name, cred)?;
    let msg = format!(
        "Merge branch \'{}\' of {} into {}",
        branch_name, remote_name, branch_name
    );
    let status = merge::merge_commit(repo, &fetch_commit, &msg, false)?;
    Ok(status)
}
