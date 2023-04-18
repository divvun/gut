use std::fmt::Display;

use super::common;

use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
/// Make repositories that match a regex become public/private
///
/// This will show all repositories that will affected by this command
/// If you want to public repositories, it'll show a confirmation prompt
/// and You have to enter 'YES' to confirm your action
pub struct MakeArgs {
    #[arg(value_enum)]
    pub visibility: Visibility,
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Regex to filter repositories
    pub regex: Filter,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Visibility {
    #[value(name = "public")]
    Public,
    #[value(name = "private")]
    Private,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
        })
    }
}

impl Visibility {
    fn is_private(&self) -> bool {
        match self {
            Visibility::Private => true,
            Visibility::Public => false,
        }
    }
}

impl MakeArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, Some(&self.regex), &user_token)?;

        let is_private = self.visibility.is_private();

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        println!(
            "The following repos will be changed to {}:",
            self.visibility
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
