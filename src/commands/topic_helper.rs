use crate::filter::{Filter, Filterable};
use crate::github;
use crate::github::{NoReposFound, RemoteRepoWithTopics, Unauthorized};
use anyhow::{Context, Result};

pub fn query_repositories_with_topics(org: &str, token: &str) -> Result<Vec<RemoteRepoWithTopics>> {
    let result =
        github::list_org_repos_with_topics(token, org).context("When fetching repositories");
    let mut repos = match result {
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
    repos.sort();
    Ok(repos)
}

pub fn filter_repos(
    repos: &[RemoteRepoWithTopics],
    topic: Option<&String>,
    regex: Option<&Filter>,
) -> Vec<RemoteRepoWithTopics> {
    if let Some(t) = topic {
        filter_repos_with_topic(repos, t)
    } else {
        RemoteRepoWithTopics::filter_with_option(repos.to_owned(), regex)
    }
}

pub fn filter_repos_by_topics(
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
