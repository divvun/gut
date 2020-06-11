use super::common;
use crate::git;
use crate::user::User;

use crate::git::GitCredential;
use anyhow::{anyhow, Result};

use crate::filter::Filter;
use git2::BranchType;
use structopt::StructOpt;

use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;

#[derive(Debug, StructOpt)]
/// Checkout a branch all repositories that their name matches a pattern or
/// a topic
///
/// This command is able to checkout a local branch as well as a remote branch
///
/// This command is able to clone a repository if it is not on the root directory
pub struct CheckoutArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, required_unless("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[structopt(long, short)]
    /// branch name to checkout
    pub branch: String,
    #[structopt(long)]
    /// Use this option to checkout a remote banch
    ///
    /// If this option is not provided, the command will report that the target branch is remote
    /// only
    pub remote: bool,
    #[structopt(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
}

impl CheckoutArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;

        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?} or topic {:?}",
                self.organisation, self.regex, self.topic
            );
            return Ok(());
        }

        for repo in filtered_repos {
            match checkout_branch(
                &repo,
                &self.branch,
                &user,
                &"origin",
                self.remote,
                self.use_https,
            ) {
                Ok(_) => println!(
                    "Checkout branch {} of repo {:?} successfully",
                    &self.branch, repo.name
                ),
                Err(e) => println!(
                    "Failed to checkout branch {} of repo {:?} because {:?}",
                    &self.branch, repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn checkout_branch(
    repo: &RemoteRepo,
    branch: &str,
    user: &User,
    remote_name: &str,
    remote: bool,
    use_https: bool,
) -> Result<()> {
    let git_repo = try_from_one(repo.clone(), user, use_https)?;
    let git_repo = git_repo.open()?;

    if git_repo.find_branch(branch, BranchType::Local).is_ok() {
        git::checkout_local_branch(&git_repo, branch)?;
    } else if remote {
        let cred = GitCredential::from(user);
        git::checkout_remote_branch(&git_repo, branch, remote_name, Some(cred))?;
    } else {
        return Err(anyhow!("There is no local branch with name: {}.\n You can use `--remote` option to checkout a remote branch.", branch));
    };

    Ok(())
}
