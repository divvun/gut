use super::common;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
// Show all repositories that match a pattern
pub struct ShowReposArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
}

impl ShowReposArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user_token,
        )?;

        print_results(&filtered_repos);

        Ok(())
    }
}

fn print_results<T: std::fmt::Debug>(repos: &[T]) {
    for repo in repos {
        println!("{:?}", repo);
    }
}
