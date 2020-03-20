use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};
use std::convert::TryFrom;

use anyhow::{Context, Result};

use crate::filter::{Filter, Filterable};
use crate::git::branch;
use crate::git::models::GitRepo;
use crate::git::push;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateBranchArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub new_branch: String,
    #[structopt(long, short)]
    pub base_branch: Option<String>,
}

impl CreateBranchArgs {
    pub fn create_branch(&self) -> Result<()> {
        let user_token = get_user_token()?;

        let remote_repos = get_remote_repos(&user_token, &self.organisation)?;

        let filtered_repos = RemoteRepo::filter_with_option(remote_repos, &self.regex);

        let default_base_branch = &"master".to_string();
        let base_branch: &str = self.base_branch.as_ref().unwrap_or(default_base_branch);
        for repo in filtered_repos {
            let result = create_branch(repo.clone(), &self.new_branch, &base_branch);
            match result {
                Ok(_) => println!(
                    "Created branch {} for repo {} successfully",
                    self.new_branch, repo.name
                ),
                Err(e) => println!(
                    "Could not create branch {} for repo {} because {}",
                    self.new_branch, repo.name, e
                ),
            }
        }

        Ok(())
    }
}

/// We need to do following steps
/// 1. Check if the repository is already exist
/// 2. if it is not exist we need to clone it
/// 3. Check out the base branch
/// 4. Create new_branch
/// 5. Push it to origin
fn create_branch(remote_repo: RemoteRepo, new_branch: &str, base_branch: &str) -> Result<()> {
    log::debug!(
        "Create new branch {} base on {} for: {:?}",
        new_branch,
        base_branch,
        remote_repo
    );

    let git_repo = GitRepo::try_from(remote_repo)?;

    let cloned_repo = git_repo.open()?;
    log::debug!("Cloned repo: {:?}", cloned_repo.path());

    branch::create_branch(&cloned_repo, new_branch, base_branch)?;
    log::debug!(
        "Create new branch {} for repo {:?} success",
        new_branch,
        cloned_repo.path()
    );

    push::push_branch(&cloned_repo, new_branch, "origin")?;

    Ok(())
}

fn get_user_token() -> Result<String> {
    super::User::get_token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn get_remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match github::list_org_repos(token, org).context("Fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<NoReposFound>() {
                anyhow::bail!("No repositories found");
            }
            if let Some(_) = e.downcast_ref::<Unauthorized>() {
                anyhow::bail!("User token invalid. Run dadmin init with a valid token");
            }
            Err(e)
        }
    }
}
