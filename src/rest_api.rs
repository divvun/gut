use super::api;
use super::api::RemoteRepo;
use anyhow::{Context, Result};
use reqwest::{blocking as req, StatusCode};
use serde::Serialize;

fn patch<T: Serialize + ?Sized>(
    url: &str,
    body: &T,
    token: &str,
) -> Result<req::Response, reqwest::Error> {
    let client = req::Client::new();
    client
        .patch(url)
        .bearer_auth(token)
        .header("User-Agent", "dadmin")
        .header("Accept", "application/vnd.github.v3+json")
        .json(body)
        .send()
}

#[derive(Serialize, Debug)]
struct DefaultBranch {
    default_branch: String,
}

pub fn set_default_branch(repo: &RemoteRepo, branch: &str, token: &str) -> Result<()> {
    let url = format!("https://api.github.com/repos/{}/{}", repo.owner, repo.name);
    let body = DefaultBranch {
        default_branch: branch.to_string(),
    };
    let response = patch(&url, &body, token)?;
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(api::Unauthorized.into());
    }

    if !status.is_success() {
        return Err(api::Unsuccessful(status).into());
    }

    Ok(())
}
