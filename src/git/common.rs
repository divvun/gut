use super::models::GitCredential;
use git2::Error;
use git2_credentials::ui4dialoguer::CredentialUI4Dialoguer;
use git2_credentials::CredentialHandler;
use git2_credentials::CredentialUI;

pub fn create_remote_callback(
    cred: &Option<GitCredential>,
) -> Result<git2::RemoteCallbacks, Error> {
    let mut cb = git2::RemoteCallbacks::new();
    let git_config = git2::Config::open_default()?;

    let credential_ui: Box<dyn CredentialUI> = match cred {
        Some(gc) => Box::new(gc.clone()),
        _ => Box::new(CredentialUI4Dialoguer {}),
    };

    // Prepare callbacks.
    let mut ch = CredentialHandler::new_with_ui(git_config, credential_ui);

    cb.credentials(move |url, username, allowed| ch.try_next_credential(url, username, allowed));

    Ok(cb)
}

pub fn ref_by_branch(branch: &str) -> String {
    format!("refs/heads/{}:refs/heads/{}", branch, branch)
}
