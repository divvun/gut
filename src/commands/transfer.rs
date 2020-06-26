use super::common;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Transfer repositories that match a regex to another organisation
///
/// This will show all repositories that will affected by this command
/// You have to enter 'YES' to confirm your action
pub struct TransferArgs {
    #[structopt(long, short)]
    /// The current organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Regex to filter repositories
    pub regex: Filter,
    /// New organisation name
    #[structopt(long, short)]
    pub new_org: String,
}

impl TransferArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos = common::query_and_filter_repositories(
            &organisation,
            Some(&self.regex),
            &user_token,
        )?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        println!(
            "The following repos will be transfered to {}:",
            self.new_org
        );

        for repo in &filtered_repos {
            println!("{}", repo.full_name());
        }

        if !confirm(filtered_repos.len(), &self.new_org)? {
            println!("Command is aborted. Nothing change!");
            return Ok(());
        }

        for repo in filtered_repos {
            let result = github::transfer_repo(&repo, &self.new_org, &user_token);
            match result {
                Ok(_) => println!(
                    "Transfer repo {} to {} successfully",
                    repo.name, self.new_org
                ),
                Err(e) => println!(
                    "Failed to Transfer repo {} to {:?} because {:?}",
                    repo.name, self.new_org, e
                ),
            }
        }

        Ok(())
    }
}

fn confirm(count: usize, org: &str) -> Result<bool> {
    let key = "YES";
    common::confirm(
        &format!(
            "Are you sure you want to transfer {} repo(s) to {}?\nEnter {} to continue",
            count, org, key
        ),
        key,
    )
}
