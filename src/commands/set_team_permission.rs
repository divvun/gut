use super::common;
use crate::github;

use anyhow::Result;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SetTeamPermissionArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub team_slug: String,
    #[structopt(long, short)]
    pub permission: String,
}

impl SetTeamPermissionArgs {
    pub fn set_permission(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, &self.regex, &user_token)?;

        for repo in filtered_repos {
            let result = github::set_team_permission(
                &self.organisation,
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
        }

        Ok(())
    }
}
