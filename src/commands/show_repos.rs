use super::common;
use crate::filter::Filter;
use crate::github::{self, RemoteRepo};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Table, format, row};
use rayon::prelude::*;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Parser)]
/// Show all repositories that match a pattern
pub struct ShowReposArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
    #[arg(long, short)]
    /// Output as JSON
    pub json: bool,
    #[arg(long, short)]
    /// Show default branch (slower due to additional GitHub API calls)
    pub default_branch: bool,
}

impl ShowReposArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::user_token()?;
        let root = common::root()?;

        let organizations = if self.all_orgs {
            let orgs = common::get_all_organizations()?;
            if orgs.is_empty() {
                println!("No organizations found in root directory");
                return Ok(());
            }
            orgs
        } else {
            vec![common::organisation(self.organisation.as_deref())?]
        };

        if self.json {
            let mut all_repos = Vec::new();
            for org in &organizations {
                if let Ok(repos) = self.show_org(org, &user_token, &root, true) {
                    all_repos.extend(repos);
                }
            }
            print_json(&all_repos)?;
        } else {
            for org in &organizations {
                let _ = self.show_org(org, &user_token, &root, false);
            }
        }

        Ok(())
    }

    fn show_org(
        &self,
        organisation: &str,
        user_token: &str,
        root: &str,
        json_mode: bool,
    ) -> anyhow::Result<Vec<RemoteRepo>> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("Querying GitHub for {} repos...", organisation));
        spinner.enable_steady_tick(Duration::from_millis(100));

        let repos = match common::query_and_filter_repositories(
            organisation,
            self.regex.as_ref(),
            user_token,
        ) {
            Ok(repos) => {
                spinner.finish_and_clear();
                if !json_mode {
                    println!("\n=== {} ===", organisation);
                }
                repos
            }
            Err(e) => {
                spinner.finish_and_clear();
                println!("Could not fetch repositories for {}: {:?}", organisation, e);
                return Ok(Vec::new());
            }
        };

        if !json_mode {
            if repos.is_empty() {
                println!("No repositories match the pattern");
            } else {
                print_table(&repos, organisation, root, user_token, self.default_branch)?;
            }
        }

        Ok(repos)
    }
}

fn print_json(repos: &[RemoteRepo]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(repos)?;
    println!("{}", json);
    Ok(())
}

fn is_cloned_locally(owner: &str, repo_name: &str, root: &str) -> bool {
    let repo_path = Path::new(root).join(owner).join(repo_name);
    repo_path.exists() && repo_path.join(".git").exists()
}

fn print_table(
    repos: &[RemoteRepo],
    owner: &str,
    root: &str,
    token: &str,
    show_default_branch: bool,
) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);

    let mut cloned_count = 0;

    if show_default_branch {
        // Fetch all default branches in parallel using rayon
        let repo_data: Vec<_> = repos
            .par_iter()
            .map(|repo| {
                let is_cloned = is_cloned_locally(owner, &repo.name, root);
                let default_branch =
                    github::default_branch(repo, token).unwrap_or_else(|_| "N/A".to_string());
                (repo, is_cloned, default_branch)
            })
            .collect();

        table.set_titles(row!["Repository", "Default Branch", "Cloned"]);

        for (repo, is_cloned, default_branch) in repo_data {
            if is_cloned {
                cloned_count += 1;
            }

            let cloned_status = if is_cloned { "Yes" } else { "No" };
            table.add_row(row![repo.name, default_branch, cloned_status]);
        }

        // Add separator and summary row
        table.add_empty_row();
        table.add_row(row![
            format!("Summary for: {}", owner),
            repos.len().to_string(),
            cloned_count.to_string()
        ]);
    } else {
        // Without default branch - faster, only 2 columns
        table.set_titles(row!["Repository", "Cloned"]);

        for repo in repos {
            let is_cloned = is_cloned_locally(owner, &repo.name, root);
            if is_cloned {
                cloned_count += 1;
            }

            let cloned_status = if is_cloned { "Yes" } else { "No" };
            table.add_row(row![repo.name, cloned_status]);
        }

        // Add separator and summary row
        table.add_empty_row();
        table.add_row(row![
            format!("Summary for: {}", owner),
            format!("{}/{}", cloned_count, repos.len())
        ]);
    }

    table.printstd();
    Ok(())
}
