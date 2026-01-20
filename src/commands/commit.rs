use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use prettytable::{Cell, Row, Table, cell, format, row};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
/// Add all and then commit with the provided messages for all
/// repositories that match a pattern or a topic
pub struct CommitArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_orgs")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    /// topic to filter
    pub topic: Option<String>,
    #[arg(long, short)]
    /// Commit message
    pub message: String,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl CommitArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_orgs(
            self.all_orgs,
            self.owner.as_deref(),
            |org| self.run_for_organization(org),
            "Committed",
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<OrgResult> {
        let user = common::user()?;
        let root = common::root()?;

        let repo_dirs = common::get_repo_dirs(
            organisation,
            self.topic.as_ref(),
            self.regex.as_ref(),
            &user.token,
            &root,
        )?;

        if repo_dirs.is_empty() {
            println!(
                "There are no repositories in {} that match the pattern {:?}",
                organisation, self.regex
            );
            return Ok(OrgResult::new(organisation));
        }

        let statuses = common::process_with_progress(
            "Committing",
            &repo_dirs,
            |dir| commit(dir, &self.message),
            |s| s.repo.clone(),
        );

        summarize(&statuses);

        let successful = statuses.iter().filter(|s| s.is_success()).count();
        let failed = statuses.iter().filter(|s| s.has_error()).count();

        Ok(OrgResult {
            org_name: organisation.to_string(),
            total_repos: repo_dirs.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
    }
}

fn commit(dir: &PathBuf, msg: &str) -> Status {
    let repo_name = path::dir_name(dir).unwrap_or_default();
    let result = || -> Result<CommitResult> {
        let git_repo =
            git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
        do_commit(&git_repo, msg)
    };
    Status {
        repo: repo_name,
        result: result(),
    }
}

fn do_commit(git_repo: &git2::Repository, msg: &str) -> Result<CommitResult> {
    let status = git::status(git_repo, true)?;

    if !status.can_commit() {
        return Ok(CommitResult::Conflict);
    }

    if !status.should_commit() {
        return Ok(CommitResult::NoChanges);
    }

    let mut index = git_repo.index()?;

    let addable_list = status.addable_list();
    for p in addable_list {
        let path = Path::new(&p);
        index.add_path(path)?;
    }

    for p in status.deleted {
        let path = Path::new(&p);
        index.remove_path(path)?;
    }

    git::commit_index(git_repo, &mut index, msg)?;

    Ok(CommitResult::Success)
}

pub enum CommitResult {
    Conflict,
    NoChanges,
    Success,
}

struct Status {
    repo: String,
    result: Result<CommitResult>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![
            cell!(b -> &self.repo),
            self.status_cell(),
            self.error_cell(),
        ])
    }

    fn status_cell(&self) -> Cell {
        match &self.result {
            Ok(r) => match r {
                CommitResult::Conflict => cell!(Fy -> "Conflict"),
                CommitResult::NoChanges => cell!("No Changes"),
                CommitResult::Success => cell!(Fg -> "Success"),
            },
            Err(_) => cell!(Fr -> "Failed"),
        }
    }

    fn error_cell(&self) -> Cell {
        match &self.result {
            Ok(CommitResult::Conflict) => {
                cell!(Fy -> "Fix conflicts and commit manually")
            }
            Err(e) => {
                let msg = format!("{}", e);
                let lines = common::sub_strings(&msg, 50);
                cell!(Fr -> lines.join("\n"))
            }
            _ => cell!(""),
        }
    }

    fn has_error(&self) -> bool {
        self.result.is_err()
    }

    fn is_success(&self) -> bool {
        matches!(&self.result, Ok(CommitResult::Success))
    }

    fn has_conflict(&self) -> bool {
        matches!(&self.result, Ok(CommitResult::Conflict))
    }
}

fn to_table(statuses: &[Status]) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status", "Error"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn summarize(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let successes: Vec<_> = statuses.iter().filter(|s| s.is_success()).collect();
    let conflicts: Vec<_> = statuses.iter().filter(|s| s.has_conflict()).collect();
    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nCommitted {} repos!", successes.len());
        println!("{}", msg.green());
    }

    if !conflicts.is_empty() {
        let msg = format!("\n{} repos have conflicts.", conflicts.len());
        println!("{}", msg.yellow());
    }

    if errors.is_empty() {
        println!("\nThere are no errors!");
    } else {
        let msg = format!("\nThere were {} errors.", errors.len());
        println!("{}", msg.red());
    }
}
