use super::common;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Make repositories that match a regex become public/private
pub struct MakeArgs {
    #[structopt(flatten)]
    pub visibility: Visibility,
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Regex to filter repositories
    pub regex: Filter,
}

#[derive(Debug, StructOpt)]
pub enum Visibility {
    #[structopt(name = "public")]
    Public,
    #[structopt(name = "private")]
    Private,
}


impl MakeArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        let is_private = match self.visibility {
            Visibility::Private => true,
            Visibility::Public => false,
        };

        for repo in filtered_repos {
            let result = github::set_repo_visibility(&repo, is_private, &user_token);
            match result {
                Ok(_) => println!(
                    "Make repo {} to {:?} successfully",
                    repo.name, self.visibility
                ),
                Err(e) => println!(
                    "Failed to make repo {} to {:?} because {:?}",
                    repo.name, self.visibility, e
                ),
            }
        }
        Ok(())
    }
}
