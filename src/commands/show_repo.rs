use super::common;
use crate::github;
use crate::github::models::Unsuccessful;
use anyhow::Result;
use clap::Parser;
use prettytable::{Cell, Row, Table, format, row};
use reqwest::StatusCode;
use std::collections::HashSet;

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

        match github::get_repo_collaborators(&organisation, repo_name, &user_token, None) {
            Ok(collaborators) => {
                let direct_users: HashSet<String> = match github::get_repo_collaborators(
                    &organisation,
                    repo_name,
                    &user_token,
                    Some("direct"),
                ) {
                    Ok(direct) => direct.into_iter().map(|c| c.login).collect(),
                    Err(_) => HashSet::new(),
                };
                let outside_users: HashSet<String> = match github::get_repo_collaborators(
                    &organisation,
                    repo_name,
                    &user_token,
                    Some("outside"),
                ) {
                    Ok(outside) => outside.into_iter().map(|c| c.login).collect(),
                    Err(_) => HashSet::new(),
                };
                print_collaborators(&collaborators, &direct_users, &outside_users);
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

fn print_collaborators(
    collaborators: &[github::RepoCollaborator],
    direct_users: &HashSet<String>,
    outside_users: &HashSet<String>,
) {
    if collaborators.is_empty() {
        println!("No collaborators have access to this repository");
        return;
    }

    let mut sorted: Vec<_> = collaborators.iter().collect();
    sorted.sort_by_key(|c| permission_rank(c.permissions.to_permission_string()));

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Username", "Permission", "Affiliation"]);

    for collaborator in &sorted {
        let permission = collaborator.permissions.to_permission_string();
        let permission_cell = permission_cell(permission);

        let affiliation = if outside_users.contains(&collaborator.login) {
            "outside"
        } else if direct_users.contains(&collaborator.login) {
            "direct"
        } else {
            "org"
        };

        let affiliation_cell = match affiliation {
            "outside" => Cell::new(affiliation).style_spec("Fr"),
            "direct" => Cell::new(affiliation).style_spec("Fc"),
            _ => Cell::new(affiliation),
        };

        table.add_row(Row::new(vec![
            Cell::new(&collaborator.login),
            permission_cell,
            affiliation_cell,
        ]));
    }

    println!("Collaborators:");
    table.printstd();
    println!("{} collaborators", collaborators.len());
    println!();
    println!("Affiliation key:");
    println!("  org     - org member, access granted through organisation or team membership");
    println!(
        "  direct  - org member, explicitly added to this repository (e.g. for elevated permissions)"
    );
    println!(
        "  outside - not an org member, explicitly added to this repository as an outside collaborator"
    );
}

fn permission_rank(permission: &str) -> u8 {
    match permission {
        "admin" => 0,
        "maintain" => 1,
        "write" => 2,
        "triage" => 3,
        "read" => 4,
        _ => 5,
    }
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
