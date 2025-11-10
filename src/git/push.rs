use super::common;
use super::models::GitCredential;
use git2::{BranchType, Error, Remote, Repository};

pub fn push_branch(
    repo: &Repository,
    branch: &str,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    let mut origin = repo.find_remote(remote_name)?;

    let remote_callbacks = common::create_remote_callback(cred)?;

    let mut po = git2::PushOptions::new();
    po.remote_callbacks(remote_callbacks);

    origin.push(&[&common::ref_by_branch(branch)], Some(&mut po))?;

    Ok(())
}

pub fn push(
    repo: &Repository,
    remote: &mut Remote,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    let remote_callbacks = common::create_remote_callback(cred)?;

    let mut po = git2::PushOptions::new();
    po.remote_callbacks(remote_callbacks);

    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .unwrap()
        .map(|a| a.unwrap())
        .map(|(a, _)| a.name().unwrap().unwrap().to_string())
        .collect();

    log::debug!("Branches {:?}", branches);

    let refs: Vec<String> = branches.iter().map(|a| common::ref_by_branch(a)).collect();

    let result = remote.push(&refs, Some(&mut po));
    log::debug!("Push result {:?}", result);
    Ok(())
}
