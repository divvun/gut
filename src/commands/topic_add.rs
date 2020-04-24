use super::common;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Add topics for all repositories that match a regex
pub struct TopicAddArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// All topics will be added
    pub topics: Vec<String>,
}

impl TopicAddArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user_token,
        )?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        for repo in filtered_repos {
            let result = add_topics(&repo, &self.topics, &user_token);
            match result {
                Ok(topics) => {
                    println!("Add topics for repo {} successfully", repo.name);
                    println!("List of topics for {} is: {:?}", repo.name, topics);
                }
                Err(e) => println!(
                    "Failed to add topics for repo {} because {:?}",
                    repo.name, e
                ),
            }
        }
        Ok(())
    }
}

fn add_topics(repo: &github::RemoteRepo, topics: &[String], token: &str) -> Result<Vec<String>> {
    let current_topics = github::get_topics(repo, token)?;
    let mut temp = vec![];
    temp.push(current_topics);
    temp.push(topics.to_owned());

    let new_topics: Vec<String> = temp.into_iter().flatten().collect();

    github::set_topics(&repo, &new_topics, &token)
}
