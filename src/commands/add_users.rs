use super::common;
use crate::github;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct AddUsersArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short, default_value = "member")]
    pub role: String,
    #[structopt(long, short)]
    pub users: Vec<String>,
}

impl AddUsersArgs {
    pub fn add_users_to_org(&self) -> Result<()> {
        let user_token = common::get_user_token()?;

        let users: Vec<String> = self.users.iter().map(|s| s.to_string()).collect();

        let results = add_list_user_to_org(&self.organisation, &self.role, users, &user_token);

        print_results(&results, &self.organisation, &self.role);

        Ok(())
    }
}

pub fn add_list_user_to_org(
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

fn print_results(results: &[(String, Result<()>)], org: &str, role: &str) {
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
