use super::common;
use crate::github;
use anyhow::{anyhow, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Show all users in an organisation
pub struct ShowUsersArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short, default_value = "all", parse(try_from_str = parse_role))]
    /// Filter members returned by their role.
    ///
    /// Can be one of:
    /// * all - All members of the organization, regardless of role.
    /// * admin - Organization owners.
    /// * member - Non-owner organization members.
    pub role: String,
}

impl ShowUsersArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let result = github::get_org_members(&self.organisation, &self.role, &user_token);

        match result {
            Ok(users) => print_results(&users),
            Err(e) => println!("Show users failed because {:?}", e),
        }

        Ok(())
    }
}

fn print_results(users: &[github::OrgMember]) {
    println!("List of users: ");
    for user in users {
        println!("{:?}", user.login);
    }
}

fn parse_role(src: &str) -> Result<String> {
    let roles = ["all", "admin", "member"];
    let src = src.to_lowercase();
    if roles.contains(&src.as_str()) {
        return Ok(src);
    }

    Err(anyhow!("role must be one of {:?}", roles))
}
