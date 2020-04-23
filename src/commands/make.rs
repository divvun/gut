use super::common;
use anyhow::Result;
use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Visibility {
    #[structopt(name = "public")]
    Public,
    #[structopt(name = "private")]
    Private,
}

#[derive(Debug, StructOpt)]
pub struct MakeArgs {
    #[structopt(flatten)]
    pub visibility: Visibility,
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
}

impl MakeArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, self.regex.as_ref(), &user_token)?;

        Ok(())
    }
}
