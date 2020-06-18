use super::common;
use crate::user::User;
use colored::*;
use prettytable::{cell, format, row, Cell, Row, Table};

use crate::git;
use anyhow::{Context, Error, Result};

use crate::filter::Filter;
use crate::git::push;
use crate::git::GitCredential;
use structopt::StructOpt;

use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use rayon::prelude::*;

#[derive(Debug, StructOpt)]
/// Push the provided branch to remote server for all repositories that match a pattern
/// or a topic
///
/// This command will do nothing if there is nothing to push
pub struct PushArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// topic to filter
    pub topic: Option<String>,
    #[structopt(long, short, default_value = "master")]
    pub branch: String,
    #[structopt(long, short)]
    pub use_https: bool,
}

impl PushArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;

        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        let statuses: Vec<_> = filtered_repos
            .par_iter()
            .map(|r| push_branch(&r, &self.branch, &user, &"origin", self.use_https))
            .collect();

        summarize(&statuses, &self.branch);

        Ok(())
    }
}

fn summarize(statuses: &[Status], branch: &str) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let success_create: Vec<_> = statuses.iter().filter(|s| s.success()).collect();

    if !success_create.is_empty() {
        let msg = format!(
            "\nPushed branch {} for {} repos!",
            branch,
            success_create.len()
        );
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!("There {} errors when process command:", errors.len());
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

fn push_branch(
    repo: &RemoteRepo,
    branch: &str,
    user: &User,
    remote_name: &str,
    use_https: bool,
) -> Status {
    log::info!("Processing repo {}", repo.name);

    let mut push_status = PushStatus::No;

    let mut push = || -> Result<()> {
        let git_repo = try_from_one(repo.clone(), user, use_https)?;
        let git_repo = git_repo
            .open()
            .with_context(|| format!("{:?} is not a git directory.", git_repo.local_path))?;

        let status = git::status(&git_repo, false)?;

        if !status.should_push() {
            push_status = PushStatus::No;
            return Ok(());
        }

        let cred = GitCredential::from(user);
        push::push_branch(&git_repo, branch, remote_name, Some(cred))?;
        push_status = PushStatus::Success(status.is_ahead);
        Ok(())
    };

    let result = push();
    if let Err(e) = result {
        push_status = PushStatus::Failed(e);
    }

    Status {
        repo: repo.clone(),
        status: push_status,
    }
}

struct Status {
    repo: RemoteRepo,
    status: PushStatus,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo.name), self.status.to_cell()])
    }

    fn has_error(&self) -> bool {
        matches!(self.status, PushStatus::Failed(_))
    }

    fn success(&self) -> bool {
        matches!(self.status, PushStatus::Success(_))
    }

    fn to_error_row(&self) -> Row {
        let e = if let PushStatus::Failed(e) = &self.status {
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

enum PushStatus {
    No,
    Success(usize),
    Failed(Error),
}

impl PushStatus {
    fn to_cell(&self) -> Cell {
        match &self {
            PushStatus::No => cell!(r -> "-"),
            PushStatus::Success(_) => cell!(Fgr -> "Success"),
            PushStatus::Failed(_) => cell!(Frr -> "Failed"),
        }
    }
}

fn to_table(statuses: &[Status]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status"]);
    table
}
