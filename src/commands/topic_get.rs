use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use prettytable::{Table, format, row};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Get topics for all repositories that match a regex
pub struct TopicGetArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

struct RepoTopics {
    repo_name: String,
    topics: Vec<String>,
}

impl TopicGetArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Retrieved",
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
                let result = github::get_topics(repo, &user_token);
                match result {
                    Ok(topics) => Ok(RepoTopics {
                        repo_name: repo.name.clone(),
                        topics,
                    }),
                    Err(e) => {
                        println!(
                            "Failed to get topics for repo {} because {:?}",
                            repo.name, e
                        );
                        Err(())
                    }
                }
            })
            .collect();

        let successful = results.iter().filter(|r| r.is_ok()).count();
        let failed = results.len() - successful;

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        table.set_titles(row!["Repository", "Topic"]);

        let mut topic_count = 0;
        for repo_topics in results.iter().flatten() {
            for (i, topic) in repo_topics.topics.iter().enumerate() {
                let repo_col = if i == 0 { &repo_topics.repo_name } else { "" };
                table.add_row(row![repo_col, topic]);
                topic_count += 1;
            }
        }

        if topic_count > 0 {
            table.printstd();
            println!(
                "{} topics across {} repos in {}",
                topic_count, successful, owner
            );
        } else {
            println!("No topics found for repos in {}", owner);
        }

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: results.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
    }
}
