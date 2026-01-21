use super::common::{self, OrgResult};
use super::models::Script;
use super::topic_helper;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github::RemoteRepoWithTopics;
use crate::user::User;
use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use std::process::Output;

/// Apply a script to all repositories that has a topics that match a pattern
/// Or to all repositories that has a specific topic
#[derive(Debug, Parser)]
pub struct TopicApplyArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    /// regex pattern to filter topics. This is required unless topic is provided.
    #[arg(long, short, required_unless_present("topic"))]
    pub regex: Option<Filter>,
    /// A topic to filter repositories. This is required unless regex is provided.
    #[arg(long, short, required_unless_present("regex"))]
    pub topic: Option<String>,
    /// The script will be applied for all repositories that match
    #[arg(long, short)]
    pub script: Script,
    /// use https to clone repositories if needed
    #[arg(long, short)]
    pub use_https: bool,
    #[arg(long, short, alias = "all-orgs")]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

impl TopicApplyArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Applied",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        println!("Topic apply for owner: {}", owner);

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        let user = common::user()?;

        let repos = topic_helper::query_repositories_with_topics(owner, &user.token)?;
        let repos =
            topic_helper::filter_repos_by_topics(&repos, self.topic.as_ref(), self.regex.as_ref());

        println!("repos {:?}", repos);

        let results: Vec<_> = repos
            .par_iter()
            .map(
                |repo| match apply(repo, script_path, &user, self.use_https) {
                    Ok(_) => {
                        println!("Apply success for repo {}", repo.repo.name);
                        true
                    }
                    Err(e) => {
                        println!("Apply failed for repo {} because {:?}", repo.repo.name, e);
                        false
                    }
                },
            )
            .collect();

        let successful = results.iter().filter(|&&success| success).count();
        let failed = results.len() - successful;

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: results.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
    }
}

fn apply(
    repo: &RemoteRepoWithTopics,
    script_path: &str,
    user: &User,
    use_https: bool,
) -> Result<Output> {
    let git_repo = try_from_one(repo.repo.clone(), user, use_https)?;

    let cloned_repo = git_repo.open_or_clone()?;
    log::debug!("Cloned repo: {:?}", cloned_repo.path());

    common::apply_script(&git_repo.local_path, script_path)
}
