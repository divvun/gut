use super::common;
use crate::filter::Filter;
use crate::github::RemoteRepo;
use clap::Parser;
use prettytable::{Table, format, row};
use std::path::Path;

#[derive(Debug, Parser)]
/// Show all repositories that match a pattern
pub struct ShowReposArgs {
    #[arg(long, short, conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
    #[arg(long, short)]
    /// Output as JSON
    pub json: bool,
    #[arg(long, short)]
    /// Show default branch
    pub default_branch: bool,
}

impl ShowReposArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::user_token()?;
        let root = common::root()?;

        let owners = if self.all_owners {
            let all_owners = common::get_all_owners()?;
            if all_owners.is_empty() {
                println!("No owners found in root directory");
                return Ok(());
            }
            all_owners
        } else {
            vec![common::owner(self.owner.as_deref())?]
        };

        if self.json {
            let mut all_repos = Vec::new();
            for org in &owners {
                if let Ok(repos) = self.show_owner(org, &user_token, &root, true) {
                    all_repos.extend(repos);
                }
            }
            print_json(&all_repos)?;
        } else {
            for org in &owners {
                let _ = self.show_owner(org, &user_token, &root, false);
            }
        }

        Ok(())
    }

    fn show_owner(
        &self,
        organisation: &str,
        user_token: &str,
        root: &str,
        json_mode: bool,
    ) -> anyhow::Result<Vec<RemoteRepo>> {
        let repos = match common::query_and_filter_repositories(
            organisation,
            self.regex.as_ref(),
            user_token,
        ) {
            Ok(repos) => repos,
            Err(e) => {
                println!("Could not fetch repositories for {}: {:?}", organisation, e);
                return Ok(Vec::new());
            }
        };

        if !json_mode {
            if repos.is_empty() {
                println!("No repositories match the pattern");
            } else {
                print_table(&repos, organisation, root, self.default_branch)?;
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
    show_default_branch: bool,
) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);

    let mut cloned_count = 0;

    if show_default_branch {
        table.set_titles(row!["Repository", "Default Branch", "Cloned"]);

        for repo in repos {
            let is_cloned = is_cloned_locally(owner, &repo.name, root);
            if is_cloned {
                cloned_count += 1;
            }

            let default_branch = repo.default_branch.as_deref().unwrap_or("N/A");
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
        // Without default branch - only 2 columns
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

    print_titled_table(owner, &table);
    Ok(())
}

fn print_titled_table(title: &str, table: &Table) {
    let table_str = table.to_string();

    // Measure rendered width
    let width = table_str
        .lines()
        // .map(|l| UnicodeWidthStr::width(l))
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);

    // Top border
    println!("+{}+", "-".repeat(width - 2));

    // Centered title row
    println!("|{:^inner_width$}|", title, inner_width = width - 2);

    println!("{}", table_str);
}
