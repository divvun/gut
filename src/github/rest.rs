use super::models;
use super::models::RemoteRepo;
use anyhow::Result;
use reqwest::{blocking as req, StatusCode};
use serde::{Deserialize, Serialize};

fn patch<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
    token: &str,
) -> Result<req::Response, reqwest::Error> {
    log::debug!("Patch: {}", url);
    let client = req::Client::new();
    client
        .patch(url)
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .header("Accept", "application/vnd.github.v3+json")
        .json(body)
        .send()
}

fn get(url: &str, token: &str, accept: Option<&str>) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    let accept = accept.unwrap_or("application/vnd.github.v3+json");
    log::debug!("PUT: {} with accept: {}", url, accept);
    client
        .get(url)
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .header("Accept", accept)
        .send()
}

fn put<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
    token: &str,
    accept: Option<&str>,
) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    let accept = accept.unwrap_or("application/vnd.github.v3+json");
    log::debug!("PUT: {} with accept: {}", url, accept);
    client
        .put(url)
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .header("Accept", accept)
        .json(body)
        .send()
}

fn post<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
    token: &str,
) -> Result<req::Response, reqwest::Error> {
    log::debug!("POST: {}", url);
    let client = req::Client::new();
    client
        .post(url)
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .header("Accept", "application/vnd.github.v3+json")
        .json(body)
        .send()
}

fn delete(url: &str, token: &str) -> Result<req::Response, reqwest::Error> {
    log::debug!("DELETE: {}", url);
    let client = req::Client::new();
    client
        .delete(url)
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
}

#[derive(Serialize, Debug)]
struct UpdateRepoBody {
    default_branch: Option<String>,
    private: Option<bool>,
    description: Option<String>,
    homepage: Option<String>,
}

impl UpdateRepoBody {
    fn default_branch(branch: &str) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: Some(branch.to_string()),
            private: None,
            description: None,
            homepage: None,
        }
    }

    fn repo_visibility(is_private: bool) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: None,
            private: Some(is_private),
            description: None,
            homepage: None,
        }
    }

    fn metadata(des: Option<&str>, homepage: Option<&str>) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: None,
            private: None,
            description: des.map(|s| s.to_string()),
            homepage: homepage.map(|s| s.to_string()),
        }
    }
}

pub fn set_default_branch(repo: &RemoteRepo, branch: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let body = UpdateRepoBody::default_branch(branch);
    let response = patch(&url, &body, token)?;

    process_response(&response).map(|_| ())
}

pub fn set_repo_visibility(repo: &RemoteRepo, is_private: bool, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let body = UpdateRepoBody::repo_visibility(is_private);
    let response = patch(&url, &body, token)?;

    process_response(&response).map(|_| ())
}

pub fn set_repo_metadata(
    repo: &RemoteRepo,
    des: Option<&str>,
    homepage: Option<&str>,
    token: &str,
) -> Result<()> {
    let url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let body = UpdateRepoBody::metadata(des, homepage);
    let response = patch(&url, &body, token)?;

    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct ProtectedBranch {
    required_status_checks: Option<RequiredStatusCheck>,
    enforce_admins: bool,
    required_pull_request_reviews: Option<RequiredPullRequestReviews>,
    restrictions: Option<Restrictions>,
    required_linear_history: bool,
    allow_force_pushes: bool,
    allow_deletions: bool,
}

#[derive(Serialize, Debug)]
struct RequiredStatusCheck {
    strict: bool,
    context: Vec<String>,
}

#[derive(Serialize, Debug)]
struct RequiredPullRequestReviews {
    dismiss_stale_reviews: bool,
    require_code_owner_reviews: bool,
    required_approving_review_count: i32,
}

#[derive(Serialize, Debug)]
struct Restrictions {
    users: Vec<String>,
    teams: Vec<String>,
    apps: Vec<String>,
}

pub fn set_protected_branch(repo: &RemoteRepo, branch: &str, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}/protection",
        repo.owner, repo.name, branch
    );
    let body = ProtectedBranch {
        required_status_checks: None,
        enforce_admins: true,
        required_pull_request_reviews: None,
        restrictions: None,
        required_linear_history: true,
        allow_force_pushes: false,
        allow_deletions: false,
    };

    log::debug!("Body {:?}", body);

    let response = put(
        &url,
        &body,
        token,
        Some("application/vnd.github.luke-cage-preview+json"),
    )?;

    log::debug!("Response: {:?}", response);

    process_response(&response).map(|_| ())
}

pub fn create_team(
    org: &str,
    team: &str,
    description: &str,
    maintainers: Vec<String>,
    is_secret: bool,
    token: &str,
) -> Result<CreateTeamResponse> {
    let url = format!("https://api.github.com/orgs/{}/teams", org);
    let privacy = if is_secret {
        "secret".to_string()
    } else {
        "closed".to_string()
    };
    let body = CreateTeamBody {
        name: team.to_string(),
        description: description.to_string(),
        maintainers,
        privacy,
    };
    log::debug!("Body {:?}", body);

    let response = post(&url, &body, token)?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: CreateTeamResponse = response.json()?;
    Ok(response_body)
}

#[derive(Serialize, Debug)]
struct CreateTeamBody {
    name: String,
    description: String,
    maintainers: Vec<String>,
    privacy: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateTeamResponse {
    pub id: i32,
    pub html_url: String,
}

pub fn remove_user_from_org(org: &str, user: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/orgs/{}/memberships/{}", org, user);

    let response = delete(&url, token)?;

    process_response(&response).map(|_| ())
}

pub fn remove_user_from_team(org: &str, team: &str, user: &str, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/memberships/{}",
        org, team, user
    );

    let response = delete(&url, token)?;

    process_response(&response).map(|_| ())
}

// https://developer.github.com/v3/teams/members/#add-or-update-team-membership
pub fn add_user_to_team(org: &str, team: &str, role: &str, user: &str, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/memberships/{}",
        org, team, user
    );

    let body = AddUserToOrgBody {
        role: role.to_string(),
    };

    let response = put(&url, &body, token, None)?;

    process_response(&response).map(|_| ())
}

pub fn invite_user_to_org(org: &str, role: &str, email: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/orgs/{}/invitations", org);

    let body = InviteUserToOrgBody {
        email: email.to_string(),
        role: role.to_string(),
        team_ids: vec![],
    };

    let response = post(&url, &body, token)?;

    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct InviteUserToOrgBody {
    email: String,
    role: String,
    team_ids: Vec<String>,
}

pub fn add_user_to_org(org: &str, role: &str, user: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/orgs/{}/memberships/{}", org, user);

    let body = AddUserToOrgBody {
        role: role.to_string(),
    };

    let response = put(&url, &body, token, None)?;

    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct AddUserToOrgBody {
    role: String,
}

pub fn create_discusstion(
    org: &str,
    team: &str,
    title: &str,
    body: &str,
    private: bool,
    token: &str,
) -> Result<CreateDiscussionResponse> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/discussions",
        org, team
    );

    let body = CreateDiscussionBody {
        title: title.to_string(),
        body: body.to_string(),
        private,
    };

    let response = post(&url, &body, token)?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: CreateDiscussionResponse = response.json()?;
    Ok(response_body)
}

#[derive(Serialize, Debug)]
struct CreateDiscussionBody {
    title: String,
    body: String,
    private: bool,
}

#[derive(Deserialize, Debug)]
pub struct CreateDiscussionResponse {
    pub html_url: String,
}

pub fn set_team_permission(
    org: &str,
    team: &str,
    owner: &str,
    repo: &str,
    permission: &str,
    token: &str,
) -> Result<()> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/repos/{}/{}",
        org, team, owner, repo
    );

    let body = SetTeamPermissionBody {
        permission: permission.to_string(),
    };

    let response = put(&url, &body, token, None)?;

    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct SetTeamPermissionBody {
    permission: String,
}

pub fn create_org_repo(
    org: &str,
    name: &str,
    public: bool,
    token: &str,
) -> Result<CreateRepoResponse> {
    let url = format!("https://api.github.com/orgs/{}/repos", org);

    let body = CreateRepoBody {
        name: name.to_string(),
        private: !public,
    };

    let response = post(&url, &body, token)?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: CreateRepoResponse = response.json()?;
    Ok(response_body)
}

#[derive(Serialize, Debug)]
struct CreateRepoBody {
    name: String,
    private: bool,
}

#[derive(Deserialize, Debug)]
pub struct CreateRepoResponse {
    pub full_name: String,
    pub html_url: String,
    pub ssh_url: String,
    pub clone_url: String,
}

pub fn delete_repo(owner: &str, repo: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let response = delete(&url, token)?;

    process_response(&response).map(|_| ())
}

// https://developer.github.com/v3/repos/#replace-all-repository-topics
pub fn set_topics(repo: &RemoteRepo, topics: &[String], token: &str) -> Result<Vec<String>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/topics",
        repo.owner, repo.name
    );

    let body = SetTopicsBody {
        names: topics.to_owned(),
    };

    let response = put(
        &url,
        &body,
        token,
        Some("application/vnd.github.mercy-preview+json"),
    )?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: TopicsResponse = response.json()?;
    Ok(response_body.names)
}

pub fn get_topics(repo: &RemoteRepo, token: &str) -> Result<Vec<String>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/topics",
        repo.owner, repo.name
    );

    let response = get(
        &url,
        token,
        Some("application/vnd.github.mercy-preview+json"),
    )?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: TopicsResponse = response.json()?;
    Ok(response_body.names)
}

#[derive(Serialize, Debug)]
pub struct SetTopicsBody {
    names: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TopicsResponse {
    names: Vec<String>,
}

pub fn transfer_repo(repo: &RemoteRepo, new_owner: &str, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/transfer",
        repo.owner, repo.name
    );
    let body = TransferBody {
        new_owner: new_owner.to_string(),
    };
    let response = post(&url, &body, token)?;
    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct TransferBody {
    new_owner: String,
}

pub fn get_public_key(repo: &RemoteRepo, token: &str) -> Result<PublicKey> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/secrets/public-key",
        repo.owner, repo.name
    );

    let response = get(&url, token, None)?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: PublicKey = response.json()?;
    Ok(response_body)
}

#[derive(Deserialize, Debug)]
pub struct PublicKey {
    pub key_id: String,
    pub key: String,
}

pub fn set_secret(
    repo: &RemoteRepo,
    name: &str,
    encrypted_value: &str,
    key_id: &str,
    token: &str,
) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/secrets/{}",
        repo.owner, repo.name, name
    );

    let body = SetSecretBody {
        encrypted_value: encrypted_value.to_string(),
        key_id: key_id.to_string(),
    };

    let response = put(&url, &body, token, None)?;
    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct SetSecretBody {
    encrypted_value: String,
    key_id: String,
}

fn process_response(response: &req::Response) -> Result<&req::Response> {
    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    Ok(response)
}
