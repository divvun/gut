use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::github;
use crate::github::rest::Label;
use anyhow::Result;
use clap::Parser;
use prettytable::{Table, format, row};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// List labels for all repositories that match a regex
pub struct LabelListArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
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
}

struct RepoLabels {
    repo_name: String,
    labels: Vec<Label>,
}

impl LabelListArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Retrieved",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        let user_token = common::user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(owner, self.regex.as_ref(), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in {} that match the pattern {:?}",
                owner, self.regex
            );
            return Ok(OrgResult::new(owner));
        }

        let results: Vec<_> = filtered_repos
            .par_iter()
            .map(|repo| {
                let result = github::get_labels(repo, &user_token);
                match result {
                    Ok(labels) => Ok(RepoLabels {
                        repo_name: repo.name.clone(),
                        labels,
                    }),
                    Err(e) => {
                        println!(
                            "Failed to get labels for repo {} because {:?}",
                            repo.name, e
                        );
                        Err(())
                    }
                }
            })
            .collect();

        let successful = results.iter().filter(|r| r.is_ok()).count();
        let failed = results.len() - successful;

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        table.set_titles(row!["Repository", "Label", "Color", "Description"]);

        let mut label_count = 0;
        for result in &results {
            if let Ok(repo_labels) = result {
                for label in &repo_labels.labels {
                    let desc = label.description.as_deref().unwrap_or("");
                    table.add_row(row![repo_labels.repo_name, label.name, label.color, desc]);
                    label_count += 1;
                }
            }
        }

        if label_count > 0 {
            table.printstd();
            println!(
                "{} labels across {} repos in {}",
                label_count, successful, owner
            );
        } else {
            println!("No labels found for repos in {}", owner);
        }

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: results.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
    }
}
