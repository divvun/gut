use super::common;
use prettytable::{cell, Cell, format, row, Row, Table};
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;
use colored::*;
use crate::commands::topic_helper;
use anyhow::{anyhow, Result, Error};

use crate::filter::Filter;
use crate::git::branch;
use crate::git::push;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Create a new branch for all repositories that match a regex or a topic
///
/// If regex is provided, this will fillter by repo name on the provided regex.
/// If topic is provided, this will fillter if a repo contains that provided topic.
/// The new branch will be based from a another branch (default is master).
/// If a matched repository is not present in root dir yet, it will be cloned.
pub struct CreateBranchArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short, required_unless("topic"))]
    pub regex: Option<Filter>,
    #[structopt(long, required_unless("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[structopt(long, short)]
    /// New branch name
    pub new_branch: String,
    #[structopt(long, short, default_value = "master")]
    /// The base branch which new branch will based of
    pub base_branch: String,
    #[structopt(long, short)]
    /// Use https to clone repositories if needed
    pub use_https: bool,
    #[structopt(long, short)]
    /// Option to push a new branch to remote after creating the new branch
    pub push: bool
}

impl CreateBranchArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;
        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();
        let statuses: Vec<_> = filtered_repos.iter().map(|r|
                create_branch(
                &r,
                &self.new_branch,
                &self.base_branch,
                &user,
                self.use_https,
                self.push,
            )
            ).collect();

        summarize(&statuses);
        Ok(())
    }
}

/// We need to do following steps
/// 1. Check if the repository is already exist
/// 2. if it is not exist we need to clone it
/// 3. Check out the base branch
/// 4. Create new_branch
/// 5. Push it to origin if needed
fn create_branch(
    remote_repo: &RemoteRepo,
    new_branch: &str,
    base_branch: &str,
    user: &User,
    use_https: bool,
    push: bool,
) -> Status {
    log::debug!(
        "Create new branch {} base on {} for: {:?}",
        new_branch,
        base_branch,
        remote_repo
    );

    let mut push_status = PushStatus::No;

    let mut create_branch = || -> Result<()> {

        let git_repo = try_from_one(remote_repo.clone(), user, use_https)?;

        let cloned_repo = git_repo.open_or_clone();
        let cloned_repo = match cloned_repo {
            Ok(repo) => repo,
            Err(e) => {
                return Err(anyhow!("Failed when open {} because {:?}", git_repo.remote_url, e));
            }
        };

        branch::create_branch(&cloned_repo, new_branch, base_branch)?;

        push_status = if push {
            match push::push_branch(&cloned_repo, new_branch, "origin", git_repo.cred) {
                Ok(_) => PushStatus::Success,
                Err(e) => PushStatus::Failed(anyhow!("Failed when push {} because {:?}", new_branch, e))
            }
        } else {
            PushStatus::No
        };

        log::debug!(
            "Create new branch {} for repo {:?} success",
            new_branch,
            cloned_repo.path()
        );


        Ok(())
    };

    let result = create_branch();

    Status {
        repo: remote_repo.clone(),
        push: push_status,
        result,
    }
}

fn summarize(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();

    if errors.is_empty() {
        println!("")
    }
}

fn to_table(statuses: &[Status]) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(
        row!["Repo", "Status", "Push"],
    );
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

struct Status {
    repo: RemoteRepo,
    push: PushStatus,
    result: Result<()>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(&self.repo.name), self.result_to_cell(), self.push.to_cell()])
    }

    fn result_to_cell(&self) -> Cell {
        match self.result {
            Ok(_) => cell!(Fg -> "Success"),
            Err(_) => cell!(Fr -> "Failed"),
        }
    }

    fn has_error(&self) -> bool {
        self.result.is_err() || self.push.is_err()
    }

}

enum PushStatus {
    Success,
    No,
    Failed(Error),
}

impl PushStatus {
    fn to_cell(&self) -> Cell {
        match &self {
            PushStatus::Success => cell!(Fgr -> "Success"),
            PushStatus::No => cell!(r -> "-"),
            PushStatus::Failed(_) => cell!(Frr -> "Failed"),
        }
    }

    fn is_err(&self) -> bool {
        matches!(*self, PushStatus::Failed(_))
    }
}
