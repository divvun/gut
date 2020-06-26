use super::common;
use crate::filter::Filter;
use crate::git;
use anyhow::Result;
use std::path::Path;
use structopt::StructOpt;

use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;
use colored::*;
use prettytable::{cell, format, row, Cell, Row, Table};
use rayon::prelude::*;

#[derive(Debug, StructOpt)]
/// Add all and then commit with the provided messages for all
/// repositories that match a pattern or a topic
pub struct CommitArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    /// topic to filter
    pub topic: Option<String>,
    #[structopt(long, short)]
    /// Commit message
    pub message: String,
    #[structopt(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
}

impl CommitArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let all_repos = topic_helper::query_repositories_with_topics(&organisation, &user.token)?;

        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?} or topic {:?}",
                organisation, self.regex, self.topic
            );
            return Ok(());
        }

        let statuses: Vec<_> = filtered_repos
            .par_iter()
            .map(|r| commit(&r, &self.message, &user, self.use_https))
            .collect();

        summarize(&statuses);

        Ok(())
    }
}

fn commit(repo: &RemoteRepo, msg: &str, user: &User, use_https: bool) -> Status {
    let commit = || -> Result<CommitResult> {
        let git_repo = try_from_one(repo.clone(), user, use_https)?;
        let git_repo = git_repo.open()?;

        let status = git::status(&git_repo, true)?;
        //let current_branch = git::head_shorthand(&git_repo)?;

        if !status.can_commit() {
            return Ok(CommitResult::Conflict);
        }

        if !status.should_commit() {
            return Ok(CommitResult::NoChanges);
        }

        let mut index = git_repo.index()?;

        let addable_list = status.addable_list();
        for p in addable_list {
            //log::debug!("addable file: {}", p);
            let path = Path::new(&p);
            index.add_path(path)?;
        }

        for p in status.deleted {
            //log::debug!("removed file: {}", p);
            let path = Path::new(&p);
            index.remove_path(path)?;
        }

        git::commit_index(&git_repo, &mut index, msg)?;

        Ok(CommitResult::Success)
    };
    Status {
        repo: repo.clone(),
        result: commit(),
    }
}

pub enum CommitResult {
    Conflict,
    NoChanges,
    Success,
}

struct Status {
    repo: RemoteRepo,
    result: Result<CommitResult>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo.name), self.status()])
    }

    fn status(&self) -> Cell {
        match &self.result {
            Ok(r) => match r {
                CommitResult::Conflict => {
                    cell!(Frl -> "There are conflicts. Fix conflicts and then commit the results.")
                }
                CommitResult::NoChanges => cell!(l -> "There is no changes."),
                CommitResult::Success => cell!(Fgl -> "Success"),
            },
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
        let msg = format!("\nDid commit for {} repos successfully!", successes.len());
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!(
            "There are {} errors when executing the command:",
            errors.len()
        );
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
