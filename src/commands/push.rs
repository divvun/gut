use super::common;
use crate::git::open;
use crate::user::User;

use anyhow::{Context, Result};

use crate::filter::Filter;
use crate::git::push;
use crate::git::GitCredential;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;

#[derive(Debug, StructOpt)]
pub struct PushArgs {
    #[structopt(long, short)]
    pub organisation: String,
    #[structopt(long, short, required_unless("topic"))]
    pub regex: Option<Filter>,
    #[structopt(long, required_unless("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[structopt(long, short)]
    pub branch: String,
    #[structopt(long, short)]
    pub use_https: bool,
}

impl PushArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;
        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        for repo in filtered_repos {
            match push_branch(&repo, &self.branch, &user, &"origin", self.use_https) {
                Ok(_) => println!(
                    "Pushed branch {} of repo {:?} successfully",
                    &self.branch, repo.name
                ),
                Err(e) => println!(
                    "Failed to push branch {} of repo {:?} because {:?}",
                    &self.branch, repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn push_branch(
    repo: &RemoteRepo,
    branch: &str,
    user: &User,
    remote_name: &str,
    use_https: bool,
) -> Result<()> {
    let git_repo = try_from_one(repo.clone(), user, use_https)?;
    let git_repo = git_repo.open()?;
    let cred = GitCredential::from(user);
    push::push_branch(&git_repo, branch, remote_name, Some(cred))?;
    Ok(())
}
