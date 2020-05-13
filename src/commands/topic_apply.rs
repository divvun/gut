use super::common;
use super::models::Script;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github;
use crate::github::{NoReposFound, RemoteRepoWithTopics, Unauthorized};
use crate::user::User;
use anyhow::{Context, Result};
use structopt::StructOpt;

/// Apply a script to all repositories that has a topics that match a pattern
/// Or to all repositories that has a specific topic
#[derive(Debug, StructOpt)]
pub struct TopicApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    /// regex pattern to filter topics. This is required unless topic is provided.
    #[structopt(long, short, required_unless("topic"))]
    pub regex: Option<Filter>,
    /// A topic to filter repositories. This is required unless regex is provided.
    #[structopt(long, short, required_unless("regex"))]
    pub topic: Option<String>,
    /// The script will be applied for all repositories that match
    #[structopt(long, short)]
    pub script: Script,
    /// use https to clone repositories if needed
    #[structopt(long, short)]
    pub use_https: bool,
}

impl TopicApplyArgs {
    pub fn run(&self) -> Result<()> {
        println!("Topic apply {:?}", self);

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        let user = common::user()?;
        let repos = query_repositories_with_topics(&self.organisation, &user.token)?;
        let repos = filter_repos(&repos, self.topic.as_ref(), self.regex.as_ref());

        println!("repos {:?}", repos);

        for repo in repos {
            match apply(&repo, &script_path, &user, self.use_https) {
                Ok(_) => println!("Apply success"),
                Err(e) => println!("Apply failed because {:?}", e),
            }
        }

        Ok(())
    }
}

fn apply(
    repo: &RemoteRepoWithTopics,
    script_path: &str,
    user: &User,
    use_https: bool,
) -> Result<()> {
    let git_repo = try_from_one(repo.repo.clone(), user, use_https)?;

    let cloned_repo = git_repo.open_or_clone()?;
    log::debug!("Cloned repo: {:?}", cloned_repo.path());

    common::apply_script(&git_repo.local_path, script_path)
}

fn query_repositories_with_topics(org: &str, token: &str) -> Result<Vec<RemoteRepoWithTopics>> {
    let repos = match github::list_org_repos_with_topics(token, org)
        .context("When fetching repositories")
    {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if e.downcast_ref::<NoReposFound>().is_some() {
                anyhow::bail!("No repositories found");
            }
            if e.downcast_ref::<Unauthorized>().is_some() {
                anyhow::bail!("User token invalid. Run `gut init` with a valid token");
            }
            Err(e)
        }
    }?;
    Ok(repos)
}

fn filter_repos(
    repos: &[RemoteRepoWithTopics],
    topic: Option<&String>,
    regex: Option<&Filter>,
) -> Vec<RemoteRepoWithTopics> {
    if let Some(t) = topic {
        filter_repos_with_topic(repos, t)
    } else {
        filter_repos_with_regex(repos, regex.unwrap())
    }
}

fn filter_repos_with_topic(
    repos: &[RemoteRepoWithTopics],
    topic: &str,
) -> Vec<RemoteRepoWithTopics> {
    repos
        .iter()
        .filter(|r| r.topics.contains(&topic.to_string()))
        .cloned()
        .collect()
}

fn filter_repos_with_regex(
    repos: &[RemoteRepoWithTopics],
    regex: &Filter,
) -> Vec<RemoteRepoWithTopics> {
    repos
        .iter()
        .filter(|r| has_pattern(r, regex))
        .cloned()
        .collect()
}

fn has_pattern(repo: &RemoteRepoWithTopics, regex: &Filter) -> bool {
    let filtered_topics: Vec<_> = repo.topics.iter().filter(|t| regex.is_match(t)).collect();
    !filtered_topics.is_empty()
}
