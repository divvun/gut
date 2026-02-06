use super::common;
use crate::github;
use crate::github::models::Unsuccessful;
use anyhow::Result;
use clap::Parser;
use prettytable::{Cell, Row, Table, format, row};
use reqwest::StatusCode;

#[derive(Debug, Parser)]
/// Show access details for a specific repository
///
/// Lists all teams and collaborators with access to the repository,
/// along with their permission levels.
pub struct ShowRepoArgs {
    #[arg(value_name = "REPO_NAME")]
    /// The repository name
    pub repo_name: String,
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: Option<String>,
}

impl ShowRepoArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::owner(self.organisation.as_deref())?;
        let repo_name = &self.repo_name;

        match github::get_repo_teams(&organisation, repo_name, &user_token) {
            Ok(teams) => {
                print_teams(&teams);
            }
            Err(e) => {
                if let Some(unsuccessful) = e.downcast_ref::<Unsuccessful>()
                    && unsuccessful.0 == StatusCode::NOT_FOUND
                {
                    println!(
                        "Could not find repository '{}/{}'. Check the name and organisation.",
                        organisation, repo_name
                    );
                    if self.organisation.is_none() {
                        println!(
                            "If this repository belongs to a different organisation, use: gut show repository {} -o <organisation>",
                            repo_name
                        );
                    }
                    return Ok(());
                }
                return Err(e);
            }
        }

        println!();

        match github::get_repo_collaborators(&organisation, repo_name, &user_token) {
            Ok(collaborators) => {
                print_collaborators(&collaborators);
            }
            Err(e) => println!("Could not fetch collaborators: {:?}", e),
        }

        Ok(())
    }
}

fn print_teams(teams: &[github::RepoTeam]) {
    if teams.is_empty() {
        println!("No teams have access to this repository");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Team Slug", "Team Name", "Permission"]);

    for team in teams {
        let permission_cell = permission_cell(&team.permission);
        table.add_row(Row::new(vec![
            Cell::new(&team.slug),
            Cell::new(&team.name),
            permission_cell,
        ]));
    }

    println!("Teams:");
    table.printstd();
    println!("{} teams", teams.len());
}

fn print_collaborators(collaborators: &[github::RepoCollaborator]) {
    if collaborators.is_empty() {
        println!("No collaborators have access to this repository");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Username", "Permission"]);

    for collaborator in collaborators {
        let permission = collaborator.permissions.to_permission_string();
        let permission_cell = permission_cell(permission);
        table.add_row(Row::new(vec![
            Cell::new(&collaborator.login),
            permission_cell,
        ]));
    }

    println!("Collaborators:");
    table.printstd();
    println!("{} collaborators", collaborators.len());
}

fn permission_cell(permission: &str) -> Cell {
    match permission {
        "admin" => Cell::new(permission).style_spec("Fy"),
        "maintain" => Cell::new(permission).style_spec("Fb"),
        "push" | "write" => Cell::new(permission).style_spec("Fg"),
        "triage" => Cell::new(permission).style_spec("Fm"),
        "pull" | "read" => Cell::new(permission).style_spec("Fc"),
        _ => Cell::new(permission).style_spec("Fr"),
    }
}
