use super::common;

use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Rename repositories that match a pattern with another pattern.
///
/// This will show all repositories that will affected by this command
/// If you want to public repositories, it'll show a confirmation prompt
/// and You have to enter 'YES' to confirm your action
pub struct RenameArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Regex to filter repositories
    pub regex: Filter,
    #[arg(long, short)]
    /// Regex to replace with
    pub new_pattern: String,
}

impl RenameArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, Some(&self.regex), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in organisation {} that match pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        println!("The following repos will be renamed");

        for repo in &filtered_repos {
            println!(
                "{} -> {}",
                repo.full_name(),
                self.regex.replace(&repo.name, &self.new_pattern)
            );
        }

        if !confirm(filtered_repos.len())? {
            println!("Command is aborted. Nothing change!");
            return Ok(());
        }

        for repo in filtered_repos {
            let new_name = self.regex.replace(&repo.name, &self.new_pattern);
            let result = github::set_repo_name(&repo, &new_name, &user_token);
            match result {
                Ok(_) => println!("Renamed repo {} to {} successfully", repo.name, new_name),
                Err(e) => println!(
                    "Failed to rename repo {} to {} because {:?}",
                    repo.name, new_name, e
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
            "Are you sure you want to rename {} repo(s)?\nEnter {} to continue",
            count, key
        ),
        key,
    )
}
