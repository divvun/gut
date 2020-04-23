use super::common;

use anyhow::Result;

use crate::convert::try_from;
use crate::filter::Filter;
use crate::git::models::GitRepo;
use crate::git::{Clonable, CloneError};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CloneArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub use_https: bool,
}

impl CloneArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let filtered_repos =
            common::query_and_filter_repositories(&self.organisation, self.regex.as_ref(), &user.token)?;

        let git_repos: Vec<GitRepo> = try_from(filtered_repos, &user, self.use_https)?;

        let results: Vec<Result<GitRepo, CloneError>> = GitRepo::gclone_list(git_repos)
            .into_iter()
            .map(|r| r.map(|(g, _)| g))
            .collect();

        print_results(&results);

        Ok(())
    }
}

fn print_results(repos: &[Result<GitRepo, CloneError>]) {
    for x in repos {
        match x {
            Ok(p) => println!("Cloned {} success at {:?}", p.remote_url, p.local_path),
            Err(e) => println!("Clone {}, failed because of {}", e.remote_url, e.source),
        }
    }
}
