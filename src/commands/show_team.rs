use super::common;
use crate::github;
use anyhow::Result;
use clap::Parser;
use colored::*;
use prettytable::{Cell, Row, Table, format, row};

#[derive(Debug, Parser)]
/// Show details of a specific team
///
/// Lists all members and repositories accessible by the team,
/// along with permission levels for repositories.
pub struct ShowTeamArgs {
    #[arg(value_name = "TEAM_SLUG")]
    /// The team slug (use `gut show teams` to list available slugs)
    pub team_slug: String,
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: Option<String>,
}

impl ShowTeamArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::owner(self.organisation.as_deref())?;
        let team_slug = &self.team_slug;

        // Get team info from the list (to get name and description)
        let teams = match github::get_teams(&organisation, &user_token) {
            Ok(teams) => teams,
            Err(e) => {
                if common::handle_org_not_found(
                    &e,
                    &format!("Could not find teams for '{}'.", organisation),
                    &format!("gut show team {} -o <organisation>", team_slug),
                    self.organisation.is_some(),
                ) {
                    return Ok(());
                }
                return Err(e);
            }
        };

        let team = teams.iter().find(|t| t.slug == *team_slug);

        match team {
            Some(team) => {
                print_team_header(team, &teams);
            }
            None => {
                println!(
                    "Team '{}' not found in organisation '{}'.",
                    team_slug, organisation
                );
                if self.organisation.is_none() {
                    println!(
                        "If this team belongs to a different organisation, use: gut show team {} -o <organisation>",
                        team_slug
                    );
                }
                println!(
                    "Use 'gut show teams -o {}' to list available teams.",
                    organisation
                );
                return Ok(());
            }
        }

        // Get and display members
        match github::get_team_members(&organisation, team_slug, &user_token) {
            Ok(members) => {
                print_members(&organisation, team_slug, &members, &user_token);
            }
            Err(e) => println!("Could not fetch team members: {:?}", e),
        }

        println!();

        // Get and display repos
        match github::get_team_repos(&organisation, team_slug, &user_token) {
            Ok(repos) => {
                print_repos(&repos);
            }
            Err(e) => println!("Could not fetch team repositories: {:?}", e),
        }

        Ok(())
    }
}

fn print_team_header(team: &github::Team, all_teams: &[github::Team]) {
    println!("Team: {} ({})", team.slug.bold().cyan(), team.name.bold());
    let description = team
        .description
        .as_ref()
        .filter(|d| !d.is_empty())
        .map(|d| d.as_str())
        .unwrap_or("");
    println!("Description: {}", description);
    if let Some(parent) = &team.parent {
        println!(
            "Parent: {} ({})",
            parent.slug.bold().cyan(),
            parent.name.bold()
        );
    }
    let children: Vec<&github::Team> = all_teams
        .iter()
        .filter(|t| t.parent.as_ref().is_some_and(|p| p.slug == team.slug))
        .collect();
    if !children.is_empty() {
        let child_list: Vec<String> = children.iter().map(|t| t.slug.clone()).collect();
        println!("Child teams: {}", child_list.join(", "));
    }
    println!();
}

fn print_members(organisation: &str, team_slug: &str, members: &[github::TeamMember], token: &str) {
    if members.is_empty() {
        println!("No members in this team");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Username", "Role"]);

    for member in members {
        let role = github::get_team_membership(organisation, team_slug, &member.login, token)
            .map(|m| m.role)
            .unwrap_or_else(|_| "unknown".to_string());

        let role_cell = match role.as_str() {
            "maintainer" => Cell::new(&role).style_spec("Fy"),
            _ => Cell::new(&role),
        };

        table.add_row(Row::new(vec![Cell::new(&member.login), role_cell]));
    }

    println!("Members:");
    table.printstd();
    println!("{} members", members.len());
}

fn print_repos(repos: &[github::TeamRepo]) {
    if repos.is_empty() {
        println!("No repositories accessible by this team");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repository", "Permission"]);

    for repo in repos {
        let permission = repo.permissions.to_permission_string();
        let permission_cell = permission_cell(permission);
        table.add_row(Row::new(vec![Cell::new(&repo.name), permission_cell]));
    }

    println!("Repositories:");
    table.printstd();
    println!("{} repositories", repos.len());
}

fn permission_cell(permission: &str) -> Cell {
    match permission {
        "admin" => Cell::new(permission).style_spec("Fy"),
        "maintain" => Cell::new(permission).style_spec("Fb"),
        "write" => Cell::new(permission).style_spec("Fg"),
        "triage" => Cell::new(permission).style_spec("Fm"),
        "read" => Cell::new(permission).style_spec("Fc"),
        _ => Cell::new(permission).style_spec("Fr"),
    }
}
