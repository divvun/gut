use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Add topics for all repositories that match a regex
pub struct TopicAddArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// All topics will be added
    pub topics: Vec<String>,
    #[arg(long, short, alias = "all-orgs")]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

impl TopicAddArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Topics Added",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        let user_token = common::user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(owner, self.regex.as_ref(), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in {} that match the pattern {:?}",
                owner, self.regex
            );
            return Ok(OrgResult::new(owner));
        }

        let results: Vec<_> = filtered_repos
            .par_iter()
            .map(|repo| {
                let result = add_topics(repo, &self.topics, &user_token);
                match result {
                    Ok(topics) => {
                        println!("Add topics for repo {} successfully", repo.name);
                        println!("List of topics for {} is: {:?}", repo.name, topics);
                        true
                    }
                    Err(e) => {
                        println!(
                            "Failed to add topics for repo {} because {:?}",
                            repo.name, e
                        );
                        false
                    }
                }
            })
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

fn add_topics(repo: &github::RemoteRepo, topics: &[String], token: &str) -> Result<Vec<String>> {
    let current_topics = github::get_topics(repo, token)?;
    let temp = vec![current_topics, topics.to_owned()];

    let new_topics: Vec<String> = temp.into_iter().flatten().collect();

    github::set_topics(repo, &new_topics, token)
}
