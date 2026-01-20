use super::common::{self, OrgResult};
use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::github::RemoteRepo;
use crate::user::User;
use anyhow::{Result, anyhow};
use clap::Parser;
use git2::BranchType;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::time::Duration;

#[derive(Debug, Parser)]
/// Checkout a branch all repositories that their name matches a pattern or
/// a topic
///
/// This command is able to checkout a local branch as well as a remote branch
///
/// This command is able to clone a repository if it is not on the root directory
pub struct CheckoutArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_orgs")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
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
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl CheckoutArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_orgs(
            self.all_orgs,
            self.owner.as_deref(),
            |org| self.run_for_organization(org),
            "Checked out",
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<OrgResult> {
        let user = common::user()?;

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message("Querying GitHub for matching repositories...");
        spinner.enable_steady_tick(Duration::from_millis(100));

        let all_repos = topic_helper::query_repositories_with_topics(organisation, &user.token)?;

        spinner.finish_and_clear();

        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in {} that match the pattern {:?} or topic {:?}",
                organisation, self.regex, self.topic
            );
            return Ok(OrgResult::new(organisation));
        }

        let total_count = filtered_repos.len();
        let results: Vec<_> = filtered_repos
            .par_iter()
            .map(|repo| {
                match checkout_branch(
                    repo,
                    &self.branch,
                    &user,
                    "origin",
                    self.remote,
                    self.use_https,
                ) {
                    Ok(_) => {
                        println!(
                            "Checkout branch {} of repo {:?} successfully",
                            &self.branch, repo.name
                        );
                        true
                    }
                    Err(e) => {
                        println!(
                            "Failed to checkout branch {} of repo {:?} because {:?}",
                            &self.branch, repo.name, e
                        );
                        false
                    }
                }
            })
            .collect();

        let success_count = results.iter().filter(|&&r| r).count();
        let fail_count = results.iter().filter(|&&r| !r).count();

        Ok(OrgResult {
            org_name: organisation.to_string(),
            total_repos: total_count,
            successful_repos: success_count,
            failed_repos: fail_count,
            dirty_repos: 0,
        })
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
