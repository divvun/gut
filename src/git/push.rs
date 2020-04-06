use super::models::GitCredential;
use git2::{Error, Remote, Repository};
use git2_credentials::ui4dialoguer::CredentialUI4Dialoguer;
use git2_credentials::CredentialHandler;
use git2_credentials::CredentialUI;

pub fn push_branch(
    repo: &Repository,
    branch: &str,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    let mut origin = repo.find_remote(remote_name)?;

    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default()?;

    let credential_ui: Box<dyn CredentialUI> = match cred {
        Some(gc) => Box::new(gc),
        _ => Box::new(CredentialUI4Dialoguer {}),
    };

    // Prepare callbacks.
    let mut ch = CredentialHandler::new_with_ui(git_config, credential_ui);

    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    let mut po = git2::PushOptions::new();
    po.remote_callbacks(cb);

    let result = origin.push(&[&ref_by_branch(branch)], Some(&mut po));
    log::debug!("Push result {:?}", result);

    Ok(())
}

fn ref_by_branch(branch: &str) -> String {
    format!("refs/heads/{}:refs/heads/{}", branch, branch)
}

pub fn push(
    repo: &Repository,
    remote: &mut Remote,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default()?;

    let credential_ui: Box<dyn CredentialUI> = match cred {
        Some(gc) => Box::new(gc),
        _ => Box::new(CredentialUI4Dialoguer {}),
    };

    // Prepare callbacks.
    let mut ch = CredentialHandler::new_with_ui(git_config, credential_ui);

    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    let mut po = git2::PushOptions::new();
    po.remote_callbacks(cb);

    let branches: Vec<String> = repo
        .branches(None)
        .unwrap()
        .map(|a| a.unwrap())
        .map(|(a, _)| a.name().unwrap().unwrap().to_string())
        .collect();

    log::debug!("Branches {:?}", branches);

    //let refs = [&ref_by_branch("master"), &ref_by_branch("new-branch")];
    let refs: Vec<String> = branches.iter().map(|a| ref_by_branch(a)).collect();

    let result = remote.push(&refs, Some(&mut po));
    log::debug!("Push result {:?}", result);
    Ok(())
}
