use super::common;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use std::fmt;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Make repositories that match a regex become public/private
///
/// This will show all repositories that will affected by this command
/// If you want to public repositories, it'll show a confirmation prompt
/// and You have to enter 'YES' to confirm your action
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

impl Visibility {
    fn is_private(&self) -> bool {
        match self {
            Visibility::Private => true,
            Visibility::Public => false,
        }
    }

    fn to_string(&self) -> String {
        match self {
            Visibility::Private => "private".to_string(),
            Visibility::Public => "public".to_string(),
        }
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl MakeArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        let is_private = self.visibility.is_private();

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        println!(
            "The following repos will be changed to {}:",
            self.visibility.to_string()
        );

        for repo in &filtered_repos {
            println!("{}", repo.full_name());
        }

        if !is_private && !confirm(filtered_repos.len())? {
            println!("Command is aborted. Nothing change!");
            return Ok(());
        }

        for repo in filtered_repos {
            let result = github::set_repo_visibility(&repo, is_private, &user_token);
            match result {
                Ok(_) => println!(
                    "Make repo {} to {} successfully",
                    repo.name,
                    self.visibility.to_string()
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

fn confirm(count: usize) -> Result<bool> {
    let key = "YES";
    common::confirm(
        &format!(
            "Are you sure you want to public {} repo(s)?\nEnter {} to continue",
            count, key
        ),
        key,
    )
}
