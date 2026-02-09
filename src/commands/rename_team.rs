use super::common;
use crate::github;
use crate::github::models::Unsuccessful;
use anyhow::Result;
use clap::Parser;
use reqwest::StatusCode;

#[derive(Debug, Parser)]
/// Rename a team (will also generate new slug)
///
/// This renames a team by its slug. The new slug will be auto-generated
/// from the new name by GitHub. You have to enter 'YES' to confirm your action.
pub struct RenameTeamArgs {
    #[arg(value_name = "TEAM_SLUG")]
    /// The current team slug (use `gut show teams` to list available slugs)
    pub team_slug: String,
    #[arg(value_name = "NEW_NAME")]
    /// The new name for the team
    pub new_name: String,
    #[arg(long, short, alias = "organisation")]
    /// Target organisation name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
}

impl RenameTeamArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let org = common::owner(self.owner.as_deref())?;

        let teams = match github::get_teams(&org, &user_token) {
            Ok(teams) => teams,
            Err(e) => {
                if let Some(unsuccessful) = e.downcast_ref::<Unsuccessful>()
                    && unsuccessful.0 == StatusCode::NOT_FOUND
                {
                    println!("Could not find teams for '{}'.", org);
                    println!("Note: Teams only exist in organisations, not personal accounts.");
                    if self.owner.is_none() {
                        println!(
                            "If you meant a different organisation, use: gut rename team {} {} -o <organisation>",
                            self.team_slug, self.new_name
                        );
                    }
                    return Ok(());
                }
                return Err(e);
            }
        };

        let team = match teams.iter().find(|t| t.slug == self.team_slug) {
            Some(t) => t,
            None => {
                println!(
                    "Team '{}' not found in organisation '{}'.",
                    self.team_slug, org
                );
                if self.owner.is_none() {
                    println!(
                        "If this team belongs to a different organisation, use: gut rename team {} {} -o <organisation>",
                        self.team_slug, self.new_name
                    );
                }
                println!("Use 'gut show teams -o {}' to list available teams.", org);
                return Ok(());
            }
        };

        println!(
            "Warning: Team references in issues, discussions, etc. may stop working after renaming."
        );
        println!();
        println!(
            "{} ({}) -> {} (slug will be auto-generated)",
            team.slug, team.name, self.new_name
        );

        if !confirm()? {
            println!("Command is aborted. Nothing change!");
            return Ok(());
        }

        match github::rename_team(&org, &self.team_slug, &self.new_name, &user_token) {
            Ok(updated) => {
                println!(
                    "Renamed team '{}' to '{}' successfully (new slug: '{}')",
                    self.team_slug, updated.name, updated.slug
                );
            }
            Err(e) => {
                println!(
                    "Failed to rename team '{}' to '{}' because {:?}",
                    self.team_slug, self.new_name, e
                );
            }
        }

        Ok(())
    }
}

fn confirm() -> Result<bool> {
    let key = "YES";
    common::confirm(
        &format!(
            "Are you sure you want to rename this team?\nEnter {} to continue",
            key
        ),
        key,
    )
}
