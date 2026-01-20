use super::common;
use crate::github;

use anyhow::{Result, anyhow};

use crate::filter::Filter;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
pub struct SetTeamPermissionArgs {
    #[arg(long, short, alias = "organisation")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Team slug
    pub team_slug: String,
    #[arg(long, short, value_parser = parse_permission)]
    /// Permission (pull | push | admin | maintain | triage) to grant the team
    pub permission: String,
}

fn parse_permission(src: &str) -> Result<String> {
    let roles = ["pull", "push", "admin", "maintain", "triage"];
    let src = src.to_lowercase();
    if roles.contains(&src.as_str()) {
        return Ok(src);
    }
    Err(anyhow!("permission must be one of {:?}", roles))
}

impl SetTeamPermissionArgs {
    pub fn set_permission(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let owner = common::owner(self.owner.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&owner, self.regex.as_ref(), &user_token)?;

        filtered_repos.par_iter().for_each(|repo| {
            let result = github::set_team_permission(
                &owner,
                &self.team_slug,
                &repo.owner,
                &repo.name,
                &self.permission,
                &user_token,
            );
            match result {
                Ok(_) => println!(
                    "Set team {} with permission {} for repo {} successfully",
                    self.team_slug, self.permission, repo.name
                ),
                Err(e) => println!(
                    "Could not set team {} with permission {} for repo {} because of {}",
                    self.team_slug, self.permission, repo.name, e
                ),
            }
        });

        Ok(())
    }
}
