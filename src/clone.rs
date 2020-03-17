use crate::api::{list_org_repos, RemoteRepo};

use anyhow::{Context, Result};

use crate::convert::try_from;
use crate::filter::{Filter, Filterable};
use crate::git::models::GitRepo;
use crate::git::{Clonable, CloneError};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CloneArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl CloneArgs {
    pub fn clone(&self) -> anyhow::Result<()> {
        let user_token = get_user_token()?;

        let remote_repos = get_remote_repos(&user_token, &self.organisation)?;

        let filtered_repos = RemoteRepo::filter_with_option(remote_repos, &self.regex);

        let git_repos: Vec<GitRepo> = try_from(filtered_repos)?;

        let results: Vec<Result<GitRepo, CloneError>> = GitRepo::gclone_list(git_repos)
            .into_iter()
            .map(|r| r.map(|(g, _)| g))
            .collect();

        print_results(&results);

        Ok(())
    }
}

fn get_user_token() -> Result<String> {
    super::User::get_token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
}

fn get_remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match list_org_repos(token, org).context("Fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if let Some(_) = e.downcast_ref::<crate::api::NoReposFound>() {
                anyhow::bail!("No repositories found");
            }
            if let Some(_) = e.downcast_ref::<crate::api::Unauthorized>() {
                anyhow::bail!("User token invalid. Run dadmin init with a valid token");
            }
            return Err(e);
        }
    }
}

fn print_results(repos: &Vec<Result<GitRepo, CloneError>>) {
    for x in repos {
        match x {
            Ok(p) => println!(
                "Cloned {} success at {}",
                p.remote_url,
                p.local_path.to_str().unwrap()
            ),
            Err(e) => println!("Clone {}, failed because of {}", e.remote_url, e.source),
        }
    }
}
