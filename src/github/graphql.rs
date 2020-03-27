use super::models::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::blocking as req;
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

fn query<T: Serialize + ?Sized>(token: &str, body: &T) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .json(body)
        .send()
}

pub fn is_valid_token(token: &str) -> anyhow::Result<String> {
    let q = UserQuery::build_query(user_query::Variables {});

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    if !response_status.is_success() {
        return Err(Unsuccessful(response_status).into());
    }

    let response_body: Response<user_query::ResponseData> = res.json()?;

    let username: &str = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .viewer
        .login
        .as_ref();
    Ok(username.to_string())
}

fn list_org_repos_rec(
    token: &str,
    org: &str,
    after: Option<String>,
) -> anyhow::Result<Vec<RemoteRepo>> {
    let q = OrganizationRepositories::build_query(organization_repositories::Variables {
        login: org.to_string(),
        after,
    });

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<organization_repositories::ResponseData> = res.json()?;

    let org_data = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .organization
        .as_ref()
        .ok_or(InvalidRepoResponse)?;

    let repositories = org_data
        .repositories
        .nodes
        .as_ref();

    let mut list_repo: Vec<RemoteRepo> = repositories
        .ok_or(NoReposFound)?
        .iter()
        .filter_map(|repo| repo.as_ref())
        .map(|x| RemoteRepo {
            name: x.name.to_string(),
            ssh_url: x.ssh_url.to_string(),
            owner: org.to_string(),
            https_url: x.url.to_string(),
        })
        .collect();

    let page_info = &org_data.repositories.page_info;

    if page_info.has_next_page {
        let after = page_info.end_cursor.as_ref().map(|x| x.to_string());
        match list_org_repos_rec(token, org, after) {
            Ok(mut l) => list_repo.append(&mut l),
            Err(e) => return Err(e),
        }
    }
    Ok(list_repo)
}

pub fn list_org_repos(token: &str, org: &str) -> anyhow::Result<Vec<RemoteRepo>> {
    list_org_repos_rec(token, org, None)
}

#[allow(dead_code)]
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
