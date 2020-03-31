use super::common;
use crate::github;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AddUsersArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short, default_value = "member")]
    pub role: String, // Fixme we can change this one to an enum Role = Owner | Member
    #[structopt(long, short)]
    pub users: Vec<String>,
    #[structopt(long, short)]
    pub team_slug: Option<String>,
}

impl AddUsersArgs {
    pub fn add_users(&self) -> Result<()> {
        match &self.team_slug {
            Some(name) => self.add_users_to_team(&name),
            None => self.add_users_to_org(),
        }
    }

    fn add_users_to_org(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = add_list_user_to_org(&self.organisation, &self.role, users, &user_token);

        print_results_org(&results, &self.organisation, &self.role);

        Ok(())
    }

    fn add_users_to_team(&self, team_name: &str) -> Result<()> {
        let user_token = common::user_token()?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = add_list_user_to_team(
            &self.organisation,
            team_name,
            &self.role,
            users,
            &user_token,
        );

        print_results_team(&results, team_name, &self.role);

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
