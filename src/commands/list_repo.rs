use super::common;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ListRepoArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl ListRepoArgs {
    pub fn show(&self) -> anyhow::Result<()> {
        let user_token = common::get_user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, &self.regex, &user_token)?;

        print_results(&filtered_repos);

        Ok(())
    }
}

fn print_results<T: std::fmt::Debug>(repos: &[T]) {
    for repo in repos {
        println!("{:?}", repo);
    }
}
