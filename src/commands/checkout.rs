use super::common;
use crate::cli::Args as CommonArgs;
use crate::git;
use crate::user::User;

use crate::git::GitCredential;
use anyhow::{Result, anyhow};

use crate::filter::Filter;
use clap::Parser;
use git2::BranchType;

use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Checkout a branch all repositories that their name matches a pattern or
/// a topic
///
/// This command is able to checkout a local branch as well as a remote branch
///
/// This command is able to clone a repository if it is not on the root directory
pub struct CheckoutArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, required_unless_present("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[arg(long, short)]
    /// branch name to checkout
    pub branch: String,
    #[arg(long)]
    /// Use this option to checkout a remote banch
    ///
    /// If this option is not provided, the command will report that the target branch is remote
    /// only
    pub remote: bool,
    #[arg(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
}

impl CheckoutArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let user = common::user()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let all_repos = topic_helper::query_repositories_with_topics(&organisation, &user.token)?;

        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in organisation {} that match the pattern {:?} or topic {:?}",
                organisation, self.regex, self.topic
            );
            return Ok(());
        }

        filtered_repos.par_iter().for_each(|repo| {
            match checkout_branch(
                repo,
                &self.branch,
                &user,
                "origin",
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
        });

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
        return Err(anyhow!(
            "There is no local branch with name: {}.\n You can use `--remote` option to checkout a remote branch.",
            branch
        ));
    };

    Ok(())
}
