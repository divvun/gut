use super::common;
use crate::github;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RemoveUsersArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub users: Vec<String>,
    #[structopt(long, short)]
    pub team_slug: Option<String>,
}

impl RemoveUsersArgs {
    pub fn remove_users(&self) -> Result<()> {
        match &self.team_slug {
            Some(name) => self.remove_users_from_team(&name),
            None => self.remove_users_from_org(),
        }
    }

    fn remove_users_from_org(&self) -> Result<()> {

        let user_token = common::get_user_token()?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = remove_list_user_from_org(&self.organisation, users, &user_token);

        print_results_org(&results, &self.organisation);

        Ok(())
    }

    fn remove_users_from_team(&self, team_name: &str) -> Result<()>{
        let user_token = common::get_user_token()?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = remove_list_user_from_team(
            &self.organisation,
            team_name,
            users,
            &user_token,
        );

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
            Ok(_) => println!(
                "Removed successfully user {} from {}",
                user, org
            ),
            Err(e) => println!(
                "Failed to remove user {} to {} because of {}",
                user, org, e
            ),
        }
    }
}

fn print_results_team(results: &[(String, Result<()>)], team: &str) {
    for (user, result) in results {
        match result {
            Ok(_) => println!(
                "Removed successfully user {} from team {}",
                user, team
            ),
            Err(e) => println!(
                "Failed to remove user {} from team {} because of {}",
                user, team, e
            ),
        }
    }
}
