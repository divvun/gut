use super::common;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use colored::*;
use prettytable::{Cell, Row, Table, format, row};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Parser)]
/// Show repositories accessible by specified user(s) in an organisation
///
/// Lists all repositories that the specified user(s) have access to,
/// along with their permission level (admin, write, read).
pub struct ShowAccessArgs {
    #[arg(value_name = "USERNAME", required = true)]
    /// One or more GitHub usernames to check
    pub users: Vec<String>,
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: String,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Long output with one row per user/repo combination
    pub long: bool,
}

#[derive(Debug, Clone)]
struct RepoPermission {
    repo_name: String,
    username: String,
    permission: String,
}

impl ShowAccessArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = &self.organisation;

        let repos = match common::query_and_filter_repositories(
            organisation,
            self.regex.as_ref(),
            &user_token,
        ) {
            Ok(repos) => repos,
            Err(e) => {
                println!("Could not fetch repositories for {}: {:?}", organisation, e);
                return Ok(());
            }
        };

        if repos.is_empty() {
            println!("No repositories match the pattern");
            return Ok(());
        }

        let repo_names: Vec<String> = repos.iter().map(|r| r.name.clone()).collect();

        // Collect permissions for all users
        let mut all_permissions: Vec<RepoPermission> = Vec::new();
        for username in &self.users {
            let permissions =
                self.get_user_permissions(username, organisation, &repo_names, &user_token);
            all_permissions.extend(permissions);
        }

        if self.long {
            self.print_long_table(organisation, &all_permissions);
        } else {
            self.print_compact_table(organisation, &repo_names, &all_permissions);
        }

        Ok(())
    }

    fn get_user_permissions(
        &self,
        username: &str,
        organisation: &str,
        repo_names: &[String],
        token: &str,
    ) -> Vec<RepoPermission> {
        let pb = indicatif::ProgressBar::new(repo_names.len() as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{prefix} {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_prefix(format!("Checking {}", username));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let results: Vec<RepoPermission> = repo_names
            .par_iter()
            .map(|repo_name| {
                let permission =
                    github::get_user_repo_permission(organisation, repo_name, username, token)
                        .unwrap_or_else(|_| "error".to_string());

                pb.set_message(repo_name.clone());
                pb.inc(1);

                RepoPermission {
                    repo_name: repo_name.clone(),
                    username: username.to_string(),
                    permission,
                }
            })
            .collect();

        pb.finish_and_clear();
        results
    }

    fn print_long_table(&self, organisation: &str, permissions: &[RepoPermission]) {
        // Sort by username, then by repo_name
        let mut sorted = permissions.to_vec();
        sorted.sort_by(|a, b| {
            a.username
                .cmp(&b.username)
                .then(a.repo_name.cmp(&b.repo_name))
        });

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        table.set_titles(row!["Repository", "User", "Access"]);

        for perm in &sorted {
            table.add_row(Row::new(vec![
                Cell::new(&perm.repo_name),
                Cell::new(&perm.username),
                self.permission_cell(&perm.permission),
            ]));
        }

        table.printstd();

        // Count unique repos
        let unique_repos: std::collections::HashSet<&str> =
            sorted.iter().map(|p| p.repo_name.as_str()).collect();
        self.print_footer(organisation, unique_repos.len());
        println!();
    }

    fn print_compact_table(
        &self,
        organisation: &str,
        repo_names: &[String],
        permissions: &[RepoPermission],
    ) {
        // Build a map: (repo, user) -> permission
        let perm_map: HashMap<(&str, &str), &str> = permissions
            .iter()
            .map(|p| {
                (
                    (p.repo_name.as_str(), p.username.as_str()),
                    p.permission.as_str(),
                )
            })
            .collect();

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BORDERS_ONLY);

        // Header row: Repository, user1, user2, ...
        let mut title_cells = vec![Cell::new("Repository")];
        for user in &self.users {
            title_cells.push(Cell::new(user));
        }
        table.set_titles(Row::new(title_cells));

        // Sort repo names
        let mut sorted_repos = repo_names.to_vec();
        sorted_repos.sort();

        for repo in &sorted_repos {
            let mut row_cells = vec![Cell::new(repo)];
            for user in &self.users {
                let permission = perm_map
                    .get(&(repo.as_str(), user.as_str()))
                    .unwrap_or(&"?");
                row_cells.push(self.permission_cell(permission));
            }
            table.add_row(Row::new(row_cells));
        }

        table.printstd();
        self.print_footer(organisation, sorted_repos.len());
        println!();
    }

    fn permission_cell(&self, permission: &str) -> Cell {
        match permission {
            "admin" => Cell::new(permission).style_spec("Fy"),
            "write" => Cell::new(permission).style_spec("Fg"),
            "read" => Cell::new(permission).style_spec("Fc"),
            "none" => Cell::new(permission).style_spec("Fr"),
            _ => Cell::new(permission).style_spec("Fr"),
        }
    }

    fn print_footer(&self, organisation: &str, count: usize) {
        let footer = match &self.regex {
            Some(filter) => format!(
                "{} repos in {} matching {}",
                count.to_string().bold(),
                organisation.bold().cyan(),
                filter.to_string().bold().cyan()
            ),
            None => format!(
                "{} repos in {}",
                count.to_string().bold(),
                organisation.bold().cyan()
            ),
        };
        println!("{}", footer);
    }
}
