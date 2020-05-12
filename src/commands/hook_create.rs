use super::common;
use super::models::Script;
use crate::github;
use crate::github::CreateHookResponse;
use std::{fmt, str::FromStr};

use crate::github::RemoteRepo;
use anyhow::{anyhow, Result};
use std::str;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
    #[structopt(long, short, required_unless("script"))]
    /// The url to which payloads will be delivered
    ///
    /// This will be overridden if script is provided
    pub url: Option<String>,
    #[structopt(long, short)]
    /// Content type, either json or form
    pub method: Method,
    #[structopt(long, short, required_unless("url"))]
    /// The script that will produce an url
    pub script: Option<Script>,
    #[structopt(long, short)]
    /// Determines what events the hook is triggered for
    pub events: Vec<String>,
}

#[derive(Debug, StructOpt)]
pub enum Method {
    #[structopt(name = "json")]
    Json,
    #[structopt(name = "form")]
    Form,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "form" {
            Ok(Method::Form)
        } else if s == "json" {
            Ok(Method::Json)
        } else {
            Err(anyhow!("Method has to be json or form"))
        }
    }
}

impl Method {
    fn value(&self) -> String {
        match self {
            Method::Json => "json".to_string(),
            Method::Form => "form".to_string(),
        }
    }
}

impl CreateArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        for repo in filtered_repos {
            match create(
                &repo,
                self.url.as_deref(),
                self.script.as_ref(),
                &self.method,
                &self.events,
                &user_token,
            ) {
                Ok(response) => println!("Success with response {:?}", response),
                Err(e) => println!("Failed because {:?}", e),
            }
        }

        Ok(())
    }
}

fn create(
    repo: &RemoteRepo,
    url: Option<&str>,
    script: Option<&Script>,
    method: &Method,
    events: &[String],
    token: &str,
) -> Result<CreateHookResponse> {
    let url = get_text(repo, url, script)?;
    github::create_hook(repo, &url, &method.to_string(), events, token)
}

fn get_text(
    repo: &RemoteRepo,
    op_text: Option<&str>,
    op_script: Option<&Script>,
) -> Result<String> {
    if let Some(script) = op_script {
        script.execute_and_get_output(&repo.name, &repo.owner)
    } else {
        op_text
            .ok_or_else(|| anyhow!("No url is provided"))
            .map(|s| s.to_string())
    }
}
