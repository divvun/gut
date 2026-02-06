use super::models;
use super::models::RemoteRepo;
use anyhow::Result;
use reqwest::{StatusCode, blocking as req};
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
        .header("User-Agent", super::USER_AGENT)
        .header("Accept", "application/vnd.github.v3+json")
        .json(body)
        .send()
}

fn get(url: &str, token: &str, accept: Option<&str>) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    let accept = accept.unwrap_or("application/vnd.github.v3+json");
    log::debug!("get: {} with accept: {}", url, accept);
    client
        .get(url)
        .bearer_auth(token)
        .header("User-Agent", super::USER_AGENT)
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
        .header("User-Agent", super::USER_AGENT)
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
        .header("User-Agent", super::USER_AGENT)
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
        .header("User-Agent", super::USER_AGENT)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
}

#[derive(Serialize, Debug)]
struct UpdateRepoBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    default_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl UpdateRepoBody {
    fn default_branch(branch: &str) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: Some(branch.to_string()),
            private: None,
            description: None,
            homepage: None,
            name: None,
        }
    }

    fn repo_visibility(is_private: bool) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: None,
            private: Some(is_private),
            description: None,
            homepage: None,
            name: None,
        }
    }

    fn metadata(des: Option<&str>, homepage: Option<&str>) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: None,
            private: None,
            description: des.map(|s| s.to_string()),
            homepage: homepage.map(|s| s.to_string()),
            name: None,
        }
    }

    fn name(name: &str) -> UpdateRepoBody {
        UpdateRepoBody {
            default_branch: None,
            private: None,
            description: None,
            homepage: None,
            name: Some(name.to_string()),
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

pub fn set_repo_name(repo: &RemoteRepo, name: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let body = UpdateRepoBody::name(name);
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

    process_response(&response).map(|_| ())
}

pub fn set_unprotected_branch(repo: &RemoteRepo, branch: &str, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}/protection",
        repo.owner, repo.name, branch
    );

    let response = delete(&url, token)?;

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

pub fn rename_team(org: &str, team_slug: &str, new_name: &str, token: &str) -> Result<Team> {
    let url = format!("https://api.github.com/orgs/{}/teams/{}", org, team_slug);
    let body = serde_json::json!({ "name": new_name });
    let response = patch(&url, &body, token)?;
    process_response(&response)?;
    response.json().map_err(Into::into)
}

pub fn get_teams(org: &str, token: &str) -> Result<Vec<Team>> {
    let url = format!("https://api.github.com/orgs/{}/teams", org);

    let response = get(&url, token, None)?;

    process_response(&response).map(|_| ())?;

    response.json().map_err(Into::into)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Team {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub parent: Option<Box<Team>>,
}

#[derive(Deserialize, Debug)]
pub struct TeamMember {
    pub login: String,
}

#[derive(Deserialize, Debug)]
pub struct TeamMembership {
    pub role: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct TeamRepo {
    pub name: String,
    pub full_name: String,
    pub permissions: TeamRepoPermissions,
}

#[derive(Deserialize, Debug)]
pub struct TeamRepoPermissions {
    pub admin: bool,
    pub maintain: bool,
    pub push: bool,
    pub triage: bool,
    pub pull: bool,
}

impl TeamRepoPermissions {
    pub fn to_permission_string(&self) -> &'static str {
        if self.admin {
            "admin"
        } else if self.maintain {
            "maintain"
        } else if self.push {
            "write"
        } else if self.triage {
            "triage"
        } else if self.pull {
            "read"
        } else {
            "none"
        }
    }
}

pub fn get_team_members(org: &str, team_slug: &str, token: &str) -> Result<Vec<TeamMember>> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/members",
        org, team_slug
    );

    let response = get(&url, token, None)?;
    process_response(&response)?;
    response.json().map_err(Into::into)
}

pub fn get_team_membership(
    org: &str,
    team_slug: &str,
    username: &str,
    token: &str,
) -> Result<TeamMembership> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/memberships/{}",
        org, team_slug, username
    );

    let response = get(&url, token, None)?;
    process_response(&response)?;
    response.json().map_err(Into::into)
}

pub fn get_team_repos(org: &str, team_slug: &str, token: &str) -> Result<Vec<TeamRepo>> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/repos",
        org, team_slug
    );

    let response = get(&url, token, None)?;
    process_response(&response)?;
    response.json().map_err(Into::into)
}

pub fn invite_user_to_org(
    org: &str,
    role: &str,
    email: &str,
    token: &str,
    teams: &[i32],
) -> Result<()> {
    let url = format!("https://api.github.com/orgs/{}/invitations", org);

    let body = InviteUserToOrgBody {
        email: email.to_string(),
        role: role.to_string(),
        team_ids: teams.to_vec(),
    };

    let response = post(&url, &body, token)?;

    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct InviteUserToOrgBody {
    email: String,
    role: String,
    team_ids: Vec<i32>,
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

pub fn create_discussion(
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

// https://developer.github.com/v3/teams/#add-or-update-team-repository-permissions
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
#[allow(dead_code)]
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

pub fn get_hooks(repo: &RemoteRepo, token: &str) -> Result<Vec<usize>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/hooks",
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

    let response_body: Vec<HookResponse> = response.json()?;
    let hooks: Vec<_> = response_body.iter().map(|r| r.id).collect();
    Ok(hooks)
}

#[derive(Deserialize, Debug)]
struct HookResponse {
    id: usize,
}

pub fn delete_hook(repo: &RemoteRepo, id: usize, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/hooks/{}",
        repo.owner, repo.name, id,
    );

    let response = delete(&url, token)?;

    process_response(&response).map(|_| ())
}

pub fn create_hook(
    repo: &RemoteRepo,
    hook_url: &str,
    content_type: &str,
    events: &[String],
    token: &str,
) -> Result<CreateHookResponse> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/hooks",
        repo.owner, repo.name
    );

    let config = CreateHookConfig {
        url: hook_url.to_string(),
        content_type: content_type.to_string(),
    };

    let body = CreateHookBody {
        config,
        events: events.to_owned(),
    };

    let response = post(&url, &body, token)?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: CreateHookResponse = response.json()?;
    Ok(response_body)
}

#[derive(Serialize, Debug)]
struct CreateHookConfig {
    url: String,
    content_type: String,
}

#[derive(Serialize, Debug)]
struct CreateHookBody {
    config: CreateHookConfig,
    events: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CreateHookResponse {
    pub id: usize,
    pub url: String,
    pub test_url: String,
    pub ping_url: String,
}

pub fn add_repo_to_team(
    repo: &RemoteRepo,
    team: &str,
    permission: &str,
    token: &str,
) -> Result<()> {
    let url = format!(
        "https://api.github.com/orgs/{}/teams/{}/repos/{}/{}",
        repo.owner, team, repo.owner, repo.name
    );
    let body = SetRepoToTeamBody {
        permission: permission.to_string(),
    };

    let response = put(&url, &body, token, None)?;

    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct SetRepoToTeamBody {
    permission: String,
}

pub fn get_repo_workflow_runs(repo: &RemoteRepo, token: &str) -> Result<Vec<Workflow>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/runs",
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

    let response_body: WorkflowResponse = response.json()?;
    //println!("repo runs {:?}", response_body);
    Ok(response_body.workflow_runs)
}

pub fn get_workflow_runs(repo: &RemoteRepo, workflow: &str, token: &str) -> Result<Vec<Workflow>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/workflows/{}/runs",
        repo.owner, repo.name, workflow
    );

    let response = get(&url, token, None)?;

    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: WorkflowResponse = response.json()?;
    println!("runs {:?}", response_body);
    Ok(response_body.workflow_runs)
}

#[derive(Deserialize, Debug)]
struct WorkflowResponse {
    _total_count: usize,
    workflow_runs: Vec<Workflow>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Workflow {
    pub id: usize,
    pub html_url: String,
    pub status: String,
}

pub fn rerun_a_workflow(repo: &RemoteRepo, id: usize, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/actions/runs/{}/rerun",
        repo.owner, repo.name, id
    );

    println!("url {}", url);

    let client = req::Client::new();
    let response = client
        .post(&url)
        .bearer_auth(token)
        .header("User-Agent", super::USER_AGENT)
        .header("Accept", "application/vnd.github.v3+json")
        .send()?;

    println!("reruns {:?}", response);
    process_response(&response).map(|_| ())
}

pub fn send_a_dispatch(repo: &RemoteRepo, token: &str) -> Result<()> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/dispatches",
        repo.owner, repo.name
    );

    println!("url {}", url);

    let body = DispatchBody {
        event_type: "repository_dispatch".to_string(),
    };

    let response = post(&url, &body, token)?;
    println!("reruns {:?}", response);
    process_response(&response).map(|_| ())
}

#[derive(Serialize, Debug)]
struct DispatchBody {
    event_type: String,
}

#[derive(Deserialize, Debug)]
pub struct RepoCollaborator {
    pub login: String,
    pub permissions: CollaboratorPermissions,
}

#[derive(Deserialize, Debug)]
pub struct CollaboratorPermissions {
    pub admin: bool,
    pub maintain: bool,
    pub push: bool,
    pub triage: bool,
    pub pull: bool,
}

impl CollaboratorPermissions {
    pub fn to_permission_string(&self) -> &'static str {
        if self.admin {
            "admin"
        } else if self.maintain {
            "maintain"
        } else if self.push {
            "write"
        } else if self.triage {
            "triage"
        } else if self.pull {
            "read"
        } else {
            "none"
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RepoTeam {
    pub slug: String,
    pub name: String,
    pub permission: String,
}

pub fn get_repo_collaborators(
    owner: &str,
    repo: &str,
    token: &str,
    affiliation: Option<&str>,
) -> Result<Vec<RepoCollaborator>> {
    let url = match affiliation {
        Some(aff) => format!(
            "https://api.github.com/repos/{}/{}/collaborators?affiliation={}",
            owner, repo, aff
        ),
        None => format!(
            "https://api.github.com/repos/{}/{}/collaborators",
            owner, repo
        ),
    };

    let response = get(&url, token, None)?;
    process_response(&response)?;
    response.json().map_err(Into::into)
}

pub fn get_repo_teams(owner: &str, repo: &str, token: &str) -> Result<Vec<RepoTeam>> {
    let url = format!("https://api.github.com/repos/{}/{}/teams", owner, repo);

    let response = get(&url, token, None)?;
    process_response(&response)?;
    response.json().map_err(Into::into)
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

#[derive(Deserialize, Debug)]
struct PermissionResponse {
    permission: String,
}

/// Get a user's permission level for a repository.
/// Returns "admin", "write", "read", or "none".
pub fn get_user_repo_permission(
    owner: &str,
    repo: &str,
    username: &str,
    token: &str,
) -> Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/collaborators/{}/permission",
        owner, repo, username
    );

    let response = get(&url, token, None)?;
    let status = response.status();

    if status == StatusCode::UNAUTHORIZED {
        return Err(models::Unauthorized.into());
    }

    // 404 means user has no direct access to the repo
    if status == StatusCode::NOT_FOUND {
        return Ok("none".to_string());
    }

    if !status.is_success() {
        return Err(models::Unsuccessful(status).into());
    }

    let response_body: PermissionResponse = response.json()?;
    Ok(response_body.permission)
}
