use super::common;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use colored::*;
use prettytable::{Table, format, row};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Show repositories accessible by specified user(s) in an organisation
///
/// Lists all repositories that the specified user(s) have access to,
/// along with their permission level (admin, write, read).
pub struct ShowUserArgs {
    #[arg(value_name = "USERNAME", required = true)]
    /// One or more GitHub usernames to check
    pub users: Vec<String>,
    #[arg(long, short)]
    /// Target organisation name
    pub organisation: String,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
}

#[derive(Debug, Clone)]
struct RepoPermission {
    repo_name: String,
    permission: String,
}

impl ShowUserArgs {
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

        for username in &self.users {
            let permissions = self.get_user_permissions(
                username,
                organisation,
                &repos.iter().map(|r| r.name.clone()).collect::<Vec<_>>(),
                &user_token,
            );

            self.print_user_table(username, organisation, &permissions);
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
                    permission,
                }
            })
            .collect();

        pb.finish_and_clear();

        // Sort by repo name and filter out "none" permissions
        let mut filtered: Vec<_> = results
            .into_iter()
            .filter(|r| r.permission != "none")
            .collect();
        filtered.sort_by(|a, b| a.repo_name.cmp(&b.repo_name));

        filtered
    }

    fn print_user_table(&self, username: &str, organisation: &str, permissions: &[RepoPermission]) {
        self.print_header(organisation);

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        table.set_titles(row!["Repository", "User", "Access"]);

        for perm in permissions {
            table.add_row(row![perm.repo_name, username, perm.permission]);
        }

        table.printstd();

        self.print_footer(username, organisation, permissions.len());

        println!();
    }

    fn print_header(&self, organisation: &str) {
        let header = match &self.regex {
            Some(filter) => format!(
                "Repos in {} matching {}",
                organisation.bold().cyan(),
                filter.to_string().bold().cyan()
            ),
            None => format!("Repos in {}", organisation.bold().cyan()),
        };
        println!("\n{}", header);
    }

    fn print_footer(&self, username: &str, organisation: &str, count: usize) {
        let footer = match &self.regex {
            Some(filter) => format!(
                "{} repos for {} in {} matching {}",
                count.to_string().bold(),
                username.bold().cyan(),
                organisation.bold().cyan(),
                filter.to_string().bold().cyan()
            ),
            None => format!(
                "{} repos for {} in {}",
                count.to_string().bold(),
                username.bold().cyan(),
                organisation.bold().cyan()
            ),
        };
        println!("{}", footer);
    }
}
