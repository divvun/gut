use graphql_client::{GraphQLQuery, Response};
use reqwest::{blocking as req, StatusCode};
use serde::Serialize;

type URI = String;
type GitSSHRemote = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct UserQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct OrganizationRepositories;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct RepositoryDefaultBranch;

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

fn query<T: Serialize + ?Sized>(token: &str, body: &T) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .json(body)
        .send()
}

pub fn is_valid_token(token: &str) -> anyhow::Result<()> {
    let q = UserQuery::build_query(user_query::Variables {});

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    if !response_status.is_success() {
        return Err(Unsuccessful(response_status).into());
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct RemoteRepo {
    pub name: String,
    pub ssh_url: GitSSHRemote,
    pub owner: String,
}

pub fn list_org_repos(token: &str, org: &str) -> anyhow::Result<Vec<RemoteRepo>> {
    let q = OrganizationRepositories::build_query(organization_repositories::Variables {
        login: org.to_string(),
    });

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<organization_repositories::ResponseData> = res.json()?;

    let repositories = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .organization
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .repositories
        .nodes
        .as_ref();

    let list_repo: Vec<RemoteRepo> = repositories
        .ok_or(NoReposFound)?
        .iter()
        .filter_map(|repo| repo.as_ref())
        .map(|x| RemoteRepo {
            name: x.name.to_string(),
            ssh_url: x.ssh_url.to_string(),
            owner: org.to_string(),
        })
        .collect();
    Ok(list_repo)
}

pub fn get_default_branch(repo: &RemoteRepo, token: &str) -> anyhow::Result<String> {
    let q = RepositoryDefaultBranch::build_query(repository_default_branch::Variables {
        owner: repo.owner.clone(),
        name: repo.name.clone(),
    });

    let response = query(token, &q)?;

    let response_status = response.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<repository_default_branch::ResponseData> = response.json()?;

    log::debug!("Response body {:?}", response_body);

    let branch: &str = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .repository
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .default_branch_ref
        .as_ref()
        .ok_or(NoDefaultBranch)?
        .name
        .as_ref();
    log::debug!("Default branch of repository {} is: {}", repo.name, branch);
    Ok(branch.to_string())
}
