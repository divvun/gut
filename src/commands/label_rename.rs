use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Rename a label for all repositories that match a regex
pub struct LabelRenameArgs {
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
    #[arg(long)]
    /// Current label name
    pub name: String,
    #[arg(long)]
    /// New label name
    pub new_name: String,
    #[arg(long, short)]
    /// Optional new color (hex code without #, e.g. "ff0000")
    pub color: Option<String>,
    #[arg(long, short)]
    /// Optional new description
    pub description: Option<String>,
}

impl LabelRenameArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Labels Renamed",
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
                let result = github::update_label(
                    repo,
                    &self.name,
                    Some(&self.new_name),
                    self.color.as_deref(),
                    self.description.as_deref(),
                    &user_token,
                );
                match result {
                    Ok(label) => {
                        println!(
                            "Renamed label '{}' to '{}' (#{}) for repo {}",
                            self.name, label.name, label.color, repo.name
                        );
                        true
                    }
                    Err(e) => {
                        println!(
                            "Failed to rename label for repo {} because {:?}",
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
