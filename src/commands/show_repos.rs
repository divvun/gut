use super::common;

use crate::filter::Filter;
use crate::github::{self, RemoteRepo};
use clap::Parser;
use prettytable::{format, row, Table};
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
}

impl ShowReposArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let root = common::root()?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        if self.json {
            print_json(&filtered_repos)?;
        } else {
            print_table(&filtered_repos, &organisation, &root, &user_token)?;
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
) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repository", "Default Branch", "Cloned Locally"]);

    let mut cloned_count = 0;

    for repo in repos {
        let is_cloned = is_cloned_locally(owner, &repo.name, root);
        if is_cloned {
            cloned_count += 1;
        }

        let cloned_status = if is_cloned { "Yes" } else { "No" };

        // Try to get default branch, use "N/A" if it fails
        let default_branch = github::default_branch(repo, token)
            .unwrap_or_else(|_| "N/A".to_string());

        table.add_row(row![repo.name, default_branch, cloned_status]);
    }

    // Add summary row
    table.add_row(row![
        format!("Summary for: {}", owner),
        repos.len().to_string(),
        cloned_count.to_string()
    ]);

    table.printstd();
    Ok(())
}
