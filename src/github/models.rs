use reqwest::StatusCode;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct RemoteRepoWithTopics {
    pub repo: RemoteRepo,
    pub topics: Vec<String>,
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
#[error("No default branch")]
pub struct NoDefaultBranch;
