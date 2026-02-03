use super::common;
use crate::github;
use crate::github::models::Unsuccessful;
use anyhow::Result;
use clap::Parser;
use prettytable::{Cell, Row, Table, format, row};
use reqwest::StatusCode;

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
            Err(e) => {
                if let Some(unsuccessful) = e.downcast_ref::<Unsuccessful>()
                    && unsuccessful.0 == StatusCode::NOT_FOUND
                {
                    println!("Could not find teams for '{}'.", organisation);
                    println!("Note: Teams only exist in organisations, not personal accounts.");
                    if self.organisation.is_none() {
                        println!(
                            "If you meant a different organisation, use: gut show teams -o <organisation>"
                        );
                    }
                    return Ok(());
                }
                println!("Show teams failed: {:?}", e);
            }
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
