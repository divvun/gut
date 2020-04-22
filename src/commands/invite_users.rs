use super::common;
use crate::github;
use std::fmt;

use anyhow::{anyhow, Result};
use std::str::FromStr;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Invite users to an organisation by emails
pub struct InviteUsersArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short, default_value)]
    /// Role of users
    /// It should be one of ["member", "admin", "billing_manager"]
    pub role: Role,
    #[structopt(long, short)]
    /// list of user's emails
    pub emails: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub enum Role {
    Member,
    Admin,
    Billing,
}

impl Role {
    fn to_value(&self) -> &str {
        match self {
            Role::Member => "direct_member",
            Role::Admin => "admin",
            Role::Billing => "billing_manager",
        }
    }

    fn to_string(&self) -> &str {
        match self {
            Role::Member => "member",
            Role::Admin => "admin",
            Role::Billing => "billing_manager",
        }
    }
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "member" {
            Ok(Role::Member)
        } else if s == "admin" {
            Ok(Role::Admin)
        } else if s == "billing_manager" {
            Ok(Role::Billing)
        } else {
            Err(anyhow!(
                "Role must be one of \"member\", \"admin\", or \"billing_manager\""
            ))
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Member
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl InviteUsersArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let emails: Vec<String> = self.emails.iter().map(|s| s.to_string()).collect();

        let results = add_list_user_to_org(
            &self.organisation,
            &self.role.to_value(),
            emails,
            &user_token,
        );

        print_results_org(&results, &self.organisation, &self.role.to_value());

        Ok(())
    }
}

fn add_list_user_to_org(
    org: &str,
    role: &str,
    emails: Vec<String>,
    token: &str,
) -> Vec<(String, Result<()>)> {
    emails
        .into_iter()
        .map(|e| (e.clone(), github::invite_user_to_org(org, role, &e, token)))
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
                "Failed to invite user {} to {} with {} role because {}",
                user, org, role, e
            ),
        }
    }
}
