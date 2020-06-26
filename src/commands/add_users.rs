use super::common;
use crate::github;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Invite users by users' usernames to an organisation
///
/// If you specify team_slug it'll try to invite users to the provided team
pub struct AddUsersArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short, default_value = "member")]
    /// Role of users
    ///
    /// It should be one of ["member", "admin"].
    ///
    /// If you specify a team role should be one of ["member", "maintainer"]
    pub role: String,
    #[structopt(long, short)]
    /// List of user's username
    pub users: Vec<String>,
    #[structopt(long, short)]
    /// Optional team slug
    pub team_slug: Option<String>,
}

impl AddUsersArgs {
    pub fn run(&self) -> Result<()> {
        match &self.team_slug {
            Some(name) => self.add_users_to_team(&name),
            None => self.add_users_to_org(),
        }
    }

    fn add_users_to_org(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = add_list_user_to_org(&organisation, &self.role, users, &user_token);

        print_results_org(&results, &organisation, &self.role);

        Ok(())
    }

    fn add_users_to_team(&self, team_name: &str) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results =
            add_list_user_to_team(&organisation, team_name, &self.role, users, &user_token);

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
