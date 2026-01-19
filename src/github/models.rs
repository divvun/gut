use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct RemoteRepo {
    pub name: String,
    pub owner: String,
    pub ssh_url: String,
    pub https_url: String,
}

impl RemoteRepo {
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

impl Ord for RemoteRepo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
impl PartialOrd for RemoteRepo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RemoteRepo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.owner == other.owner
    }
}

#[derive(Debug, Clone, Eq)]
pub struct RemoteRepoWithTopics {
    pub repo: RemoteRepo,
    pub topics: Vec<String>,
}

impl Ord for RemoteRepoWithTopics {
    fn cmp(&self, other: &Self) -> Ordering {
        self.repo.cmp(&other.repo)
    }
}
impl PartialOrd for RemoteRepoWithTopics {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RemoteRepoWithTopics {
    fn eq(&self, other: &Self) -> bool {
        self.repo == other.repo
    }
}

#[derive(thiserror::Error, Debug)]
#[error("User unauthorized")]
pub struct Unauthorized;

#[derive(thiserror::Error, Debug)]
#[error("Unsuccessful request with status code: {0}")]
pub struct Unsuccessful(pub StatusCode);

#[derive(thiserror::Error, Debug)]
#[error("invalid response when fetching repositories")]
pub struct InvalidRepoResponse;

#[derive(thiserror::Error, Debug)]
#[error("no repositories found")]
pub struct NoReposFound;

#[derive(thiserror::Error, Debug)]
#[error("no members found")]
pub struct NoMembersFound;

#[derive(thiserror::Error, Debug)]
#[error("No default branch")]
pub struct NoDefaultBranch;
