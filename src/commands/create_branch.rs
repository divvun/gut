use super::common;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;

use anyhow::Result;

use crate::filter::Filter;
use crate::git::branch;
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
    #[structopt(long, short)]
    pub use_https: bool,
}

impl CreateBranchArgs {
    pub fn create_branch(&self) -> Result<()> {
        let user_token = common::get_user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, &self.regex, &user_token)?;

        let default_base_branch = &"master".to_string();
        let base_branch: &str = self.base_branch.as_ref().unwrap_or(default_base_branch);

        for repo in filtered_repos {
            let result =
                create_branch(repo.clone(), &self.new_branch, &base_branch, self.use_https);
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
fn create_branch(
    remote_repo: RemoteRepo,
    new_branch: &str,
    base_branch: &str,
    use_https: bool,
) -> Result<()> {
    log::debug!(
        "Create new branch {} base on {} for: {:?}",
        new_branch,
        base_branch,
        remote_repo
    );

    let git_repo = try_from_one(remote_repo, use_https)?;

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
