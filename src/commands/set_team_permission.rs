use super::common;
use crate::cli::Args as CommonArgs;
use crate::github;

use anyhow::Result;

use crate::filter::Filter;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
pub struct SetTeamPermissionArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Team slug
    pub team_slug: String,
    #[arg(long, short)]
    ///The permission to grant the team on this repositories
    ///
    /// Can be one of:
    ///
    /// pull | push | admin | maintain | triage
    pub permission: String,
}

impl SetTeamPermissionArgs {
    pub fn set_permission(&self, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        filtered_repos.par_iter().for_each(|repo| {
            let result = github::set_team_permission(
                &organisation,
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
