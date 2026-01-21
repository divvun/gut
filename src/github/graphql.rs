use super::models::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::blocking as req;
use serde::Serialize;

#[allow(clippy::upper_case_acronyms)]
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
struct OwnerRepositories;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct RepositoryDefaultBranch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct OwnerRepositoriesWithTopics;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "github.graphql",
    query_path = "user_query.graphql",
    response_derives = "Debug"
)]
struct OrganizationMembers;

fn query<T: Serialize + ?Sized>(token: &str, body: &T) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .header("User-Agent", super::USER_AGENT)
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

#[derive(Debug)]
#[allow(dead_code)]
pub struct OrgMember {
    pub login: String,
    pub url: String,
    //pub role: String,
}

//#[derive(Debug)]
//pub enum OrgRole {
//Member,
//Admin
//}

pub fn get_org_members(org: &str, token: &str) -> anyhow::Result<Vec<OrgMember>> {
    get_org_members_rec(org, token, None)
}

fn get_org_members_rec(
    org: &str,
    token: &str,
    after: Option<String>,
) -> anyhow::Result<Vec<OrgMember>> {
    let q = OrganizationMembers::build_query(organization_members::Variables {
        login: org.to_string(),
        after,
    });

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<organization_members::ResponseData> = res.json()?;

    let org_data = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .organization
        .as_ref()
        .ok_or(InvalidRepoResponse)?;

    let members = org_data.members_with_role.nodes.as_ref();

    let mut list_member: Vec<OrgMember> = members
        .ok_or(NoMembersFound)?
        .iter()
        .filter_map(|user| user.as_ref())
        .map(|x| OrgMember {
            login: x.login.to_string(),
            url: x.url.to_string(),
        })
        .collect();

    let page_info = &org_data.members_with_role.page_info;

    if page_info.has_next_page {
        let after = page_info.end_cursor.as_ref().map(|x| x.to_string());
        match get_org_members_rec(org, token, after) {
            Ok(mut l) => list_member.append(&mut l),
            Err(e) => return Err(e),
        }
    }
    Ok(list_member)
}

fn list_owner_repos_rec(
    token: &str,
    owner: &str,
    after: Option<String>,
) -> anyhow::Result<Vec<RemoteRepo>> {
    let q = OwnerRepositories::build_query(owner_repositories::Variables {
        login: owner.to_string(),
        after,
    });

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<owner_repositories::ResponseData> = res.json()?;

    let owner_data = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .repository_owner
        .as_ref()
        .ok_or(InvalidRepoResponse)?;

    let repositories = owner_data.repositories.nodes.as_ref();

    let mut list_repo: Vec<RemoteRepo> = repositories
        .ok_or(NoReposFound)?
        .iter()
        .filter_map(|repo| repo.as_ref())
        .map(|x| RemoteRepo {
            name: x.name.to_string(),
            ssh_url: x.ssh_url.to_string(),
            owner: owner.to_string(),
            https_url: x.url.to_string(),
            default_branch: x.default_branch_ref.as_ref().map(|b| b.name.to_string()),
        })
        .collect();

    let page_info = &owner_data.repositories.page_info;

    if page_info.has_next_page {
        let after = page_info.end_cursor.as_ref().map(|x| x.to_string());
        match list_owner_repos_rec(token, owner, after) {
            Ok(mut l) => list_repo.append(&mut l),
            Err(e) => return Err(e),
        }
    }
    Ok(list_repo)
}

pub fn list_owner_repos(token: &str, owner: &str) -> anyhow::Result<Vec<RemoteRepo>> {
    list_owner_repos_rec(token, owner, None)
}

fn list_owner_repos_with_topics_rec(
    token: &str,
    owner: &str,
    after: Option<String>,
) -> anyhow::Result<Vec<RemoteRepoWithTopics>> {
    let q = OwnerRepositoriesWithTopics::build_query(owner_repositories_with_topics::Variables {
        login: owner.to_string(),
        after,
    });

    let res = query(token, &q)?;

    let response_status = res.status();
    if response_status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Unauthorized.into());
    }

    let response_body: Response<owner_repositories_with_topics::ResponseData> = res.json()?;

    let owner_data = response_body
        .data
        .as_ref()
        .ok_or(InvalidRepoResponse)?
        .repository_owner
        .as_ref()
        .ok_or(InvalidRepoResponse)?;

    let repositories = owner_data.repositories.nodes.as_ref();

    let temp = vec![];
    let mut list_repo: Vec<RemoteRepoWithTopics> = repositories
        .ok_or(NoReposFound)?
        .iter()
        .filter_map(|repo| repo.as_ref())
        .map(|x| RemoteRepoWithTopics {
            repo: RemoteRepo {
                name: x.name.to_string(),
                ssh_url: x.ssh_url.to_string(),
                owner: owner.to_string(),
                https_url: x.url.to_string(),
                default_branch: None,
            },
            topics: x
                .repository_topics
                .nodes
                .as_ref()
                .unwrap_or(&temp)
                .iter()
                .filter_map(|t| t.as_ref())
                .map(|x| x.topic.name.to_string())
                .collect(),
        })
        .collect();

    let page_info = &owner_data.repositories.page_info;

    if page_info.has_next_page {
        let after = page_info.end_cursor.as_ref().map(|x| x.to_string());
        match list_owner_repos_with_topics_rec(token, owner, after) {
            Ok(mut l) => list_repo.append(&mut l),
            Err(e) => return Err(e),
        }
    }
    Ok(list_repo)
}

pub fn list_owner_repos_with_topics(
    token: &str,
    owner: &str,
) -> anyhow::Result<Vec<RemoteRepoWithTopics>> {
    list_owner_repos_with_topics_rec(token, owner, None)
}

#[allow(dead_code)]
pub fn default_branch(repo: &RemoteRepo, token: &str) -> anyhow::Result<String> {
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
