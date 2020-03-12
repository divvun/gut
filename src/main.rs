mod api;
mod cli;
mod config;
mod git;
mod path;
mod toml;
mod user;

use crate::api::{list_repos, RemoteRepo};
use anyhow::{Context, Result};
use cli::{Args, Commands, Filter, InitArgs};
use std::convert::TryFrom;
use structopt::StructOpt;

use config::Config;
use git::models::GitRepo;
use git::Clonable;
use path::get_local_path;
use user::User;

fn main() -> Result<()> {
    color_backtrace::install();

    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("dadmin"), log::LevelFilter::Debug)
        .init();

    let args = Args::from_args();
    log::debug!("Arguments: {:?}", args);

    match args.command {
        Commands::Init(InitArgs { root, token }) => {
            let user = match User::new(token) {
                Ok(user) => { user },
                Err(e) => match e.downcast_ref::<api::Unauthorized>() {
                    Some(_) => anyhow::bail!("Token is invalid. Check https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"),
                    _ => return Err(e)
                }
            };
            user.save_user()?;
            let config = Config::new(root.path);
            config.save_config()
        }
        Commands::ListRepos => {
            let repos = match list_repos(
                &args.global_args.organisation,
                &args.global_args.repositories,
            )
            .context("Fetching repositories")
            {
                Ok(repos) => repos,
                Err(e) => {
                    if let Some(_) = e.downcast_ref::<api::NoReposFound>() {
                        anyhow::bail!("No repositories found");
                    }
                    if let Some(_) = e.downcast_ref::<api::Unauthorized>() {
                        anyhow::bail!("User token invalid. Run dadmin init with a valid token");
                    }
                    return Err(e);
                }
            };

            for repo in &repos {
                println!("{:?}", repo);
            }

            Ok(())
        }
        Commands::CloneRepos => clone_repositories(
            &args.global_args.organisation,
            &args.global_args.repositories,
        ),
        _ => Ok(()),
    }
}

fn clone_repositories(organisation: &str, repository_regex: &Option<Filter>) -> Result<()> {
    let remote_repos = list_repos(organisation, repository_regex)?;
    let git_repos: Vec<GitRepo> = try_from(remote_repos)?;

    let results = GitRepo::gclone_list(git_repos);

    for x in &results {
        match x {
            Ok(p) => println!(
                "Clone {} success at {}",
                p.remote_url,
                p.local_path.to_str().unwrap()
            ),
            Err(e) => println!("Clone {}, failed because of {}", e.remote_url, e.source),
        }
    }
    Ok(())
}

impl TryFrom<RemoteRepo> for GitRepo {
    type Error = std::io::Error;

    fn try_from(repo: RemoteRepo) -> Result<Self, Self::Error> {
        let local_path = get_local_path(&repo.owner, &repo.name).ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot create local path",
        ))?;
        Ok(GitRepo {
            remote_url: repo.ssh_url.clone(),
            local_path: local_path,
        })
    }
}

fn try_from<T, U: TryFrom<T>>(vec: impl IntoIterator<Item = T>) -> Result<Vec<U>, U::Error> {
    vec.into_iter().map(|t| U::try_from(t)).collect()
}
