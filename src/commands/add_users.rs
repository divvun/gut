use super::common;
use crate::github;

use anyhow::Result;

use clap::Parser;

#[derive(Debug, Parser)]
/// Add users to an organisation or team
///
/// If you specify team_slug it'll add users to the provided team instead.
///
/// This command only works with GitHub organisations, not user accounts.
pub struct AddUsersArgs {
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: Option<String>,
    #[arg(long, short, default_value = "member")]
    /// Role (member | admin) for org, or (member | maintainer) for team
    pub role: String,
    #[arg(long, short)]
    /// Usernames to add (eg: -u user1 -u user2)
    pub users: Vec<String>,
    #[arg(long, short)]
    /// Optional team slug
    pub team_slug: Option<String>,
}

impl AddUsersArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::owner(self.organisation.as_deref())?;
        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        match &self.team_slug {
            Some(team_name) => {
                let results =
                    add_list_user_to_team(&organisation, team_name, &self.role, users, &user_token);
                print_results_team(&results, team_name, &self.role);
            }
            None => {
                let results = add_list_user_to_org(&organisation, &self.role, users, &user_token);
                print_results_org(&results, &organisation, &self.role);
            }
        }

        Ok(())
    }
}

fn add_list_user_to_org(
    org: &str,
    role: &str,
    users: Vec<String>,
    token: &str,
) -> Vec<(String, Result<()>)> {
    users
        .into_iter()
        .map(|u| (u.clone(), github::add_user_to_org(org, role, &u, token)))
        .collect()
}

fn add_list_user_to_team(
    org: &str,
    team: &str,
    role: &str,
    users: Vec<String>,
    token: &str,
) -> Vec<(String, Result<()>)> {
    users
        .into_iter()
        .map(|u| {
            (
                u.clone(),
                github::add_user_to_team(org, team, role, &u, token),
            )
        })
        .collect()
}

fn print_results_org(results: &[(String, Result<()>)], org: &str, role: &str) {
    for (user, result) in results {
        match result {
            Ok(_) => println!(
                "Invited successfully user {} to {} with {} role",
                user, org, role
            ),
            Err(e) => println!(
                "Failed to invite user {} to {} with {} role because of {}",
                user, org, role, e
            ),
        }
    }
}

fn print_results_team(results: &[(String, Result<()>)], team: &str, role: &str) {
    for (user, result) in results {
        match result {
            Ok(_) => println!(
                "Invited successfully user {} to team {} with {} role",
                user, team, role
            ),
            Err(e) => println!(
                "Failed to invite user {} to team {} with {} role because of {}.\n Please notice that you need to add users to your organisation before adding them to a team.",
                user, team, role, e
            ),
        }
    }
}
