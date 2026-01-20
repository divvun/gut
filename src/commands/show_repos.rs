use super::common;

use crate::filter::Filter;
use crate::github::{self, RemoteRepo};
use clap::Parser;
use prettytable::{Table, format, row};
use rayon::prelude::*;
use std::path::Path;

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

        if self.all_orgs {
            self.show_all_orgs(&user_token, &root)
        } else {
            let organisation = common::organisation(self.organisation.as_deref())?;
            self.show_single_org(&organisation, &user_token, &root)
        }
    }

    fn show_single_org(
        &self,
        organisation: &str,
        user_token: &str,
        root: &str,
    ) -> anyhow::Result<()> {
        let filtered_repos =
            common::query_and_filter_repositories(organisation, self.regex.as_ref(), user_token)?;

        if self.json {
            print_json(&filtered_repos)?;
        } else {
            print_table(
                &filtered_repos,
                organisation,
                root,
                user_token,
                self.default_branch,
            )?;
        }

        Ok(())
    }

    fn show_all_orgs(&self, user_token: &str, root: &str) -> anyhow::Result<()> {
        let organizations = common::get_all_organizations()?;

        if organizations.is_empty() {
            println!("No organizations found in root directory");
            return Ok(());
        }

        if self.json {
            // For JSON mode, collect all repos from all orgs
            let mut all_repos = Vec::new();
            for org in &organizations {
                if let Ok(repos) =
                    common::query_and_filter_repositories(org, self.regex.as_ref(), user_token)
                {
                    all_repos.extend(repos);
                }
            }
            print_json(&all_repos)?;
        } else {
            // For table mode, print a table for each org
            for org in &organizations {
                println!("\n=== {} ===", org);
                match common::query_and_filter_repositories(org, self.regex.as_ref(), user_token) {
                    Ok(repos) => {
                        if repos.is_empty() {
                            println!("No repositories match the pattern");
                        } else {
                            print_table(&repos, org, root, user_token, self.default_branch)?;
                        }
                    }
                    Err(e) => {
                        println!("Could not fetch repositories for {}: {:?}", org, e);
                    }
                }
            }
        }

        Ok(())
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
