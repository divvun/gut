use super::common;
use crate::github;
use crate::github::models::Unsuccessful;
use anyhow::Result;
use clap::Parser;
use prettytable::{Cell, Row, Table, format, row};
use reqwest::StatusCode;
use std::collections::HashMap;

#[derive(Debug, Parser)]
/// Show all teams in an organisation
///
/// Lists all teams with their slug (used in commands) and description.
pub struct ShowTeamsArgs {
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Display teams as a tree
    pub tree: bool,
}

impl ShowTeamsArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::owner(self.organisation.as_deref())?;

        let result = github::get_teams(&organisation, &user_token);

        match result {
            Ok(teams) => {
                if self.tree {
                    print_tree(&organisation, &teams);
                } else {
                    print_results(&organisation, &teams);
                }
            }
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

fn print_tree(organisation: &str, teams: &[github::Team]) {
    if teams.is_empty() {
        println!("No teams found in {}", organisation);
        return;
    }

    // Group teams by parent slug
    let mut children_map: HashMap<Option<&str>, Vec<&github::Team>> = HashMap::new();
    for team in teams {
        let parent_slug = team.parent.as_ref().map(|p| p.slug.as_str());
        children_map.entry(parent_slug).or_default().push(team);
    }

    // Sort children alphabetically within each group
    for children in children_map.values_mut() {
        children.sort_by(|a, b| a.slug.cmp(&b.slug));
    }

    println!("Teams in {}:", organisation);

    let roots = children_map.get(&None).cloned().unwrap_or_default();
    let total = roots.len();
    for (i, team) in roots.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{} ({})", connector, team.name, team.slug);
        let prefix = if is_last { "    " } else { "│   " };
        print_children(team, prefix, &children_map);
    }

    println!();
    println!("{} teams", teams.len());
}

fn print_children(
    team: &github::Team,
    prefix: &str,
    children_map: &HashMap<Option<&str>, Vec<&github::Team>>,
) {
    if let Some(children) = children_map.get(&Some(team.slug.as_str())) {
        let total = children.len();
        for (i, child) in children.iter().enumerate() {
            let is_last = i == total - 1;
            let connector = if is_last { "└── " } else { "├── " };
            println!("{}{}{} ({})", prefix, connector, child.name, child.slug);
            let next_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            print_children(child, &next_prefix, children_map);
        }
    }
}
