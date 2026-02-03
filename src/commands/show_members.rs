use super::common;
use crate::github;
use anyhow::Result;
use clap::Parser;
use prettytable::{Cell, Row, Table, format, row};

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
        let two_factor_cell = match member.has_two_factor_enabled {
            Some(true) => Cell::new("yes").style_spec("Fg"),
            Some(false) => Cell::new("no").style_spec("Fr"),
            None => Cell::new("-"),
        };
        let role_cell = match member.role.as_str() {
            "admin" => Cell::new(&member.role).style_spec("Fy"),
            _ => Cell::new(&member.role),
        };
        table.add_row(Row::new(vec![
            Cell::new(&member.login),
            role_cell,
            two_factor_cell,
        ]));
    }

    println!("Members of {}:", organisation);
    table.printstd();

    println!("{} members", members.len());
}
