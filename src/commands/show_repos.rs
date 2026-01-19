use super::common;

use crate::filter::Filter;
use crate::github::RemoteRepo;
use clap::Parser;

#[derive(Debug, Parser)]
/// Show all repositories that match a pattern
pub struct ShowReposArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
    #[arg(long, short)]
    /// Output as JSON
    pub json: bool,
}

impl ShowReposArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        if self.json {
            print_json(&filtered_repos)?;
        } else {
            print_list(&filtered_repos);
        }

        Ok(())
    }
}

fn print_list(repos: &[RemoteRepo]) {
    for repo in repos {
        println!("{}", repo.name);
    }
}

fn print_json(repos: &[RemoteRepo]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(repos)?;
    println!("{}", json);
    Ok(())
}
