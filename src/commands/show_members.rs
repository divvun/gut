use super::common;
use crate::github;
use anyhow::Result;
use clap::Parser;
use colored::*;
use prettytable::{Table, format, row};

#[derive(Debug, Parser)]
/// Show all members in an organisation
///
/// This command only works with GitHub organisations, not user accounts.
pub struct ShowMembersArgs {
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

impl ShowMembersArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = &self.organisation;

        let result = github::get_org_members(organisation, &user_token);

        match result {
            Ok(users) => print_results(organisation, &users),
            Err(e) => println!("Show members failed because {:?}", e),
        }

        Ok(())
    }
}

fn print_results(organisation: &str, members: &[github::OrgMember]) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Username", "Role", "2FA"]);

    for member in members {
        let two_factor = match member.has_two_factor_enabled {
            Some(true) => "yes".green(),
            Some(false) => "no".red(),
            None => "-".normal(),
        };
        let role = match member.role.as_str() {
            "admin" => member.role.yellow(),
            _ => member.role.normal(),
        };
        table.add_row(row![member.login, role, two_factor]);
    }

    println!("Members of {}:", organisation);
    table.printstd();

    println!("{} members", members.len());
}

//fn parse_role(src: &str) -> Result<String> {
//let roles = ["all", "admin", "member"];
//let src = src.to_lowercase();
//if roles.contains(&src.as_str()) {
//return Ok(src);
//}

//Err(anyhow!("role must be one of {:?}", roles))
//}
