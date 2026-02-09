use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Create a label for all repositories that match a regex
pub struct LabelCreateArgs {
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
    #[arg(long, short)]
    /// Label name
    pub name: String,
    #[arg(long, short)]
    /// Label color (hex code without #, e.g. "ff0000")
    pub color: String,
    #[arg(long, short)]
    /// Optional label description
    pub description: Option<String>,
}

impl LabelCreateArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Labels Created",
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
                let result = github::create_label(
                    repo,
                    &self.name,
                    &self.color,
                    self.description.as_deref(),
                    &user_token,
                );
                match result {
                    Ok(label) => {
                        println!(
                            "Created label '{}' (#{}) for repo {}",
                            label.name, label.color, repo.name
                        );
                        true
                    }
                    Err(e) => {
                        println!(
                            "Failed to create label for repo {} because {:?}",
                            repo.name, e
                        );
                        false
                    }
                }
            })
            .collect();

        let successful = results.iter().filter(|&&success| success).count();
        let failed = results.len() - successful;

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: results.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
    }
}
