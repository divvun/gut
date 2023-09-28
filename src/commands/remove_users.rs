use crate::cli::Args as CommonArgs;
use super::common;
use crate::github;

use anyhow::Result;

use clap::Parser;

#[derive(Debug, Parser)]
/// Remove users by users' usernames from an organisation
///
/// If you specify team_slug it'll try to remove users from the provided team
pub struct RemoveUsersArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// List of user's username
    pub users: Vec<String>,
    #[arg(long, short)]
    /// Optional team slug
    pub team_slug: Option<String>,
}

impl RemoveUsersArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match &self.team_slug {
            Some(name) => self.remove_users_from_team(name, common_args),
            None => self.remove_users_from_org(common_args),
        }
    }

    fn remove_users_from_org(&self, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = remove_list_user_from_org(&organisation, users, &user_token);

        print_results_org(&results, &organisation);

        Ok(())
    }

    fn remove_users_from_team(&self, team_name: &str, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = remove_list_user_from_team(&organisation, team_name, users, &user_token);

        print_results_team(&results, team_name);

        Ok(())
    }
}

fn remove_list_user_from_org(
    org: &str,
    users: Vec<String>,
    token: &str,
) -> Vec<(String, Result<()>)> {
    users
        .into_iter()
        .map(|u| (u.clone(), github::remove_user_from_org(org, &u, token)))
        .collect()
}

fn remove_list_user_from_team(
    org: &str,
    team: &str,
    users: Vec<String>,
    token: &str,
) -> Vec<(String, Result<()>)> {
    users
        .into_iter()
        .map(|u| {
            (
                u.clone(),
                github::remove_user_from_team(org, team, &u, token),
            )
        })
        .collect()
}

fn print_results_org(results: &[(String, Result<()>)], org: &str) {
    for (user, result) in results {
        match result {
            Ok(_) => println!("Removed successfully user {} from {}", user, org),
            Err(e) => println!("Failed to remove user {} to {} because of {}", user, org, e),
        }
    }
}

fn print_results_team(results: &[(String, Result<()>)], team: &str) {
    for (user, result) in results {
        match result {
            Ok(_) => println!("Removed successfully user {} from team {}", user, team),
            Err(e) => println!(
                "Failed to remove user {} from team {} because of {}",
                user, team, e
            ),
        }
    }
}
