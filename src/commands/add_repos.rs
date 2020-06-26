use super::common;
use crate::filter::Filter;
use crate::github;

use anyhow::{anyhow, Result};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Add all matched repositories to a team by using team_slug
pub struct AddRepoArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short, default_value = "pull", parse(try_from_str = parse_permission))]
    ///The permission to grant the team on repositories
    ///
    /// Can be one of:
    ///
    /// pull | push | admin | maintain | triage
    pub permission: String,
    #[structopt(long, short)]
    /// optional team slug
    #[structopt(long, short)]
    pub team_slug: String,
}

impl AddRepoArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos = common::query_and_filter_repositories(
            &organisation,
            self.regex.as_ref(),
            &user.token,
        )?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        for repo in filtered_repos {
            let result =
                github::add_repo_to_team(&repo, &self.team_slug, &self.permission, &user.token);

            match result {
                Ok(_) => println!(
                    "Added repo {}/{} to team {} successfully",
                    repo.owner, repo.name, self.team_slug
                ),
                Err(e) => println!(
                    "Failed to add repo {}/{} to team {} because {:?}",
                    repo.owner, repo.name, self.team_slug, e
                ),
            }
        }

        Ok(())
    }
}

fn parse_permission(src: &str) -> Result<String> {
    let roles = ["pull", "push", "admin", "maintain", "triage"];
    let src = src.to_lowercase();
    if roles.contains(&src.as_str()) {
        return Ok(src);
    }

    Err(anyhow!("permission must be one of {:?}", roles))
}
