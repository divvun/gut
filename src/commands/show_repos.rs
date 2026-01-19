use super::common;

use crate::filter::Filter;
use clap::Parser;

#[derive(Debug, Parser)]
/// Show all repositories that match a pattern
pub struct ShowReposArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl ShowReposArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user_token)?;

        print_results(&filtered_repos);

        Ok(())
    }
}

fn print_results<T: std::fmt::Debug>(repos: &[T]) {
    for repo in repos {
        println!("{:?}", repo);
    }
}
