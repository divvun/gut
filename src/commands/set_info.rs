use super::common;
use crate::github;

use anyhow::Result;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set description and/or website for all local repositories that match regex
pub struct InfoArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
    #[structopt(long, short, required_unless("website"))]
    /// Description, this is required unless website is provided
    pub description: Option<String>,
    #[structopt(long, short, required_unless("description"))]
    /// Homepage, this is required unless description is provided
    pub website: Option<String>,
}

impl InfoArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        for repo in filtered_repos {
            let result = github::set_repo_metadata(
                &repo,
                self.description.as_deref(),
                self.website.as_deref(),
                &user_token,
            );
            match result {
                Ok(_) => println!("Set info for repo {} successfully", repo.name),
                Err(e) => println!("Failed to set info for repo {} because {:?}", repo.name, e),
            }
        }
        Ok(())
    }
}
