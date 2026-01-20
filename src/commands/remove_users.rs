use super::common;
use crate::github;

use anyhow::Result;

use clap::Parser;

#[derive(Debug, Parser)]
/// Remove users by users' usernames from an owner
///
/// If you specify team_slug it'll try to remove users from the provided team
pub struct RemoveUsersArgs {
    #[arg(long, short, alias = "organisation")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Usernames to remove (eg: -u user1 -u user2)
    pub users: Vec<String>,
    #[arg(long, short)]
    /// Optional team slug
    pub team_slug: Option<String>,
}

impl RemoveUsersArgs {
    pub fn run(&self) -> Result<()> {
        match &self.team_slug {
            Some(name) => self.remove_users_from_team(name),
            None => self.remove_users_from_org(),
        }
    }

    fn remove_users_from_org(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let owner = common::owner(self.owner.as_deref())?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = remove_list_user_from_org(&owner, users, &user_token);

        print_results_org(&results, &owner);

        Ok(())
    }

    fn remove_users_from_team(&self, team_name: &str) -> Result<()> {
        let user_token = common::user_token()?;
        let owner = common::owner(self.owner.as_deref())?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = remove_list_user_from_team(&owner, team_name, users, &user_token);

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
