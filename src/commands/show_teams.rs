use super::common;
use crate::github;
use anyhow::Result;
use clap::Parser;
use prettytable::{Cell, Row, Table, format, row};

#[derive(Debug, Parser)]
/// Show all teams in an organisation
///
/// Lists all teams with their slug (used in commands) and description.
pub struct ShowTeamsArgs {
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: Option<String>,
}

impl ShowTeamsArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::owner(self.organisation.as_deref())?;

        let result = github::get_teams(&organisation, &user_token);

        match result {
            Ok(teams) => print_results(&organisation, &teams),
            Err(e) => println!("Show teams failed because {:?}", e),
        }

        Ok(())
    }
}

fn print_results(organisation: &str, teams: &[github::Team]) {
    if teams.is_empty() {
        println!("No teams found in {}", organisation);
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Slug", "Name", "Description"]);

    for team in teams {
        let description = team.description.as_deref().unwrap_or("-");
        table.add_row(Row::new(vec![
            Cell::new(&team.slug),
            Cell::new(&team.name),
            Cell::new(description),
        ]));
    }

    println!("Teams in {}:", organisation);
    table.printstd();

    println!("{} teams", teams.len());
}
