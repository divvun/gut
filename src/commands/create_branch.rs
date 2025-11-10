use super::common;
use crate::cli::Args as CommonArgs;
use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;
use anyhow::{Error, Result, anyhow};
use colored::*;
use prettytable::{Cell, Row, Table, cell, format, row};

use crate::filter::Filter;
use crate::git::branch;
use crate::git::push;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Create a new branch for all repositories that match a regex or a topic
///
/// If regex is provided, this will fillter by repo name on the provided regex.
/// If topic is provided, this will fillter if a repo contains that provided topic.
/// The new branch will be based from a another branch (default is main).
/// If a matched repository is not present in root dir yet, it will be cloned.
pub struct CreateBranchArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short, required_unless_present("topic"))]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, required_unless_present("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[arg(long, short)]
    /// New branch name
    pub new_branch: String,
    #[arg(long, short, default_value = "main")]
    /// The base branch which new branch will based of
    pub base_branch: String,
    #[arg(long, short)]
    /// Use https to clone repositories if needed
    pub use_https: bool,
    #[arg(long, short)]
    /// Option to push a new branch to remote after creating the new branch
    pub push: bool,
}

impl CreateBranchArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
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
                "There is no repositories in organisation {} matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        let statuses: Vec<_> = filtered_repos
            .par_iter()
            .map(|r| {
                create_branch(
                    r,
                    &self.new_branch,
                    &self.base_branch,
                    &user,
                    self.use_https,
                    self.push,
                )
            })
            .collect();

        summarize(&statuses, &self.new_branch);
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
                return Err(anyhow!(
                    "Failed when open {} because {:?}",
                    git_repo.remote_url,
                    e
                ));
            }
        };

        branch::create_branch(&cloned_repo, new_branch, base_branch)?;

        push_status = if push {
            match push::push_branch(&cloned_repo, new_branch, "origin", git_repo.cred) {
                Ok(_) => PushStatus::Success,
                Err(e) => {
                    PushStatus::Failed(anyhow!("Failed when push {} because {:?}", new_branch, e))
                }
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

fn summarize(statuses: &[Status], branch: &str) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let success_create: Vec<_> = statuses.iter().filter(|s| s.result.is_ok()).collect();

    if !success_create.is_empty() {
        let msg = format!(
            "\nCreated new branch {} for {} repos!",
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

fn to_table(statuses: &[Status]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status", "Push"]);
    table
}

struct Status {
    repo: RemoteRepo,
    push: PushStatus,
    result: Result<()>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![
            cell!(b -> &self.repo.name),
            self.result_to_cell(),
            self.push.to_cell(),
        ])
    }

    fn to_error_row(&self) -> Row {
        let e = if let Err(e1) = &self.result {
            e1
        } else if let PushStatus::Failed(e2) = &self.push {
            e2
        } else {
            panic!("This should have an error here");
        };
        let msg = format!("{:?}", e);
        let lines = common::sub_strings(msg.as_str(), 80);
        let lines = lines.join("\n");
        row!(cell!(b -> &self.repo.name), cell!(Fr -> lines.as_str()))
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
