use super::common;

use crate::github::RemoteRepo;
use anyhow::{anyhow, Error, Result};

use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::git::models::GitRepo;
use crate::git::Clonable;
use crate::user::User;
use clap::Parser;
use colored::*;
use prettytable::{cell, format, row, Cell, Row, Table};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Clone all repositories that matches a pattern
pub struct CloneArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
}

impl CloneArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let use_https = match self.use_https {
            true => true,
            false => common::use_https()?,
        };

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user.token)?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                &organisation, self.regex
            );
            return Ok(());
        }

        let statuses: Vec<_> = filtered_repos
            .par_iter()
            .map(|r| clone(r, &user, use_https))
            .collect();

        summarize(&statuses);

        Ok(())
    }
}

fn clone(repo: &RemoteRepo, user: &User, use_https: bool) -> Status {
    let cl = || -> Result<GitRepo> {
        let git_repo = try_from_one(repo.clone(), user, use_https)?;
        if git_repo.local_path.exists() {
            return Err(anyhow!(
                "Repository {} is already exist at {:?}",
                repo.name,
                git_repo.local_path
            ));
        }
        let result = git_repo.gclone()?;
        Ok(result)
    };
    let result = cl();
    Status {
        repo: repo.clone(),
        result,
    }
}

struct Status {
    repo: RemoteRepo,
    result: Result<GitRepo, Error>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo.name), self.status()])
    }

    fn status(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Success"),
            Err(_) => cell!(Frr -> "Failed"),
        }
    }

    fn has_error(&self) -> bool {
        matches!(self.result, Err(_))
    }

    fn to_error_row(&self) -> Row {
        let e = if let Err(e) = &self.result {
            e
        } else {
            panic!("This should have an error here");
        };

        let msg = format!("{:?}", e);
        let lines = common::sub_strings(msg.as_str(), 80);
        let lines = lines.join("\n");
        row!(cell!(b -> &self.repo.name), cell!(Fr -> lines.as_str()))
    }
}

fn to_table(statuses: &[Status]) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn summarize(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nCloned {} repos successfully!", successes.len());
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!("There {} errors when cloning:", errors.len());
        println!("\n{}\n", msg.red());

        let mut error_table = Table::new();
        error_table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        error_table.set_titles(row!["Repo", "Error"]);
        for error in errors {
            error_table.add_row(error.to_error_row());
        }
        error_table.printstd();
    }
}
