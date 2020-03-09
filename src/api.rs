use crate::cli::Filter;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{blocking as req, StatusCode};

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

#[derive(thiserror::Error, Debug)]
#[error("User unauthorized")]
pub struct Unauthorized;

#[derive(thiserror::Error, Debug)]
#[error("no repositories found")]
pub struct NoReposFound;

#[derive(thiserror::Error, Debug)]
#[error("Unsuccessful request with status code: {0}")]
pub struct Unsuccessful(StatusCode);

#[derive(thiserror::Error, Debug)]
#[error("invalid response when fetching repositories")]
pub struct InvalidRepoResponse;

pub fn is_valid_token(token: &str) -> anyhow::Result<()> {
    let client = req::Client::new();
    let q = UserQuery::build_query(user_query::Variables {});

    let res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .json(&q)
        .send()?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    if !response_status.is_success() {
        return Err(Unsuccessful(response_status).into());
    }

    Ok(())
}

pub fn list_repos(
    organisation: &str,
    repository_regex: &Option<Filter>,
) -> anyhow::Result<Vec<String>> {
    let user_token = match super::User::get_token() {
        Ok(user_token) => user_token,
        Err(_) => return Err(Unauthorized.into()),
    };

    let client = req::Client::new();

    let q = OrganizationRepositories::build_query(organization_repositories::Variables {
        login: organisation.to_string(),
    });

    let res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(user_token)
        .header("User-Agent", "dadmin")
        .json(&q)
        .send()?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<organization_repositories::ResponseData> = res.json()?;

    let total_count: i64 = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .organization
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .repositories
        .total_count;

    //TODO: Implement pagination with graphql cursor magic and suffering
    anyhow::ensure!(
        total_count <= 100,
        "too many repos! Pagintation not yet implemtented."
    );

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

    let repo_names: Vec<String> = repositories
        .ok_or(NoReposFound)?
        .iter()
        .filter_map(|repo| repo.as_ref())
        .map(|x| x.name.to_string())
        .collect();

    Ok(match repository_regex {
        Some(regex) => repo_names
            .into_iter()
            .filter(|repo_name| regex.is_match(&repo_name))
            .collect(),
        None => repo_names,
    })
}
