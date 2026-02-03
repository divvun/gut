use super::common;
use crate::github;
use anyhow::Result;
use clap::Parser;
use prettytable::{Table, format, row};

#[derive(Debug, Parser)]
/// Show all members in an organisation
///
/// This command only works with GitHub organisations, not user accounts.
pub struct ShowUsersArgs {
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: String,
    //#[arg(long, short, default_value = "all", parse(try_from_str = parse_role))]
    // Filter members returned by their role.
    //
    // Can be one of:
    // * all - All members of the organisation, regardless of role.
    // * admin - Organization owners.
    // * member - Non-owner organisation members.
    //pub role: String,
}

impl ShowUsersArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = &self.organisation;

        let result = github::get_org_members(organisation, &user_token);

        match result {
            Ok(users) => print_results(organisation, &users),
            Err(e) => println!("Show users failed because {:?}", e),
        }

        Ok(())
    }
}

fn print_results(organisation: &str, users: &[github::OrgMember]) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Username", "URL"]);

    for user in users {
        table.add_row(row![user.login, user.url]);
    }

    table.add_empty_row();
    table.add_row(row!["Total", users.len()]);

    println!("Members of {}:", organisation);
    table.printstd();
}

//fn parse_role(src: &str) -> Result<String> {
//let roles = ["all", "admin", "member"];
//let src = src.to_lowercase();
//if roles.contains(&src.as_str()) {
//return Ok(src);
//}

//Err(anyhow!("role must be one of {:?}", roles))
//}
