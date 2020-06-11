use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::git::MergeStatus;
use crate::path;
use crate::user::User;
use anyhow::{Context, Error, Result};
use colored::*;
use prettytable::{cell, format, row, Cell, Row, Table};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Pull the current branch of all local repositories that match a regex
///
/// This command only works on those repositories that has been cloned in root directory
///
pub struct PullArgs {
    #[structopt(long, short)]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// Option to stash if there are unstaged changes
    pub stash: bool,
}

impl PullArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        let statuses: Vec<_> = sub_dirs
            .iter()
            .map(|d| pull(&d, &user, self.stash))
            .collect();

        summarize(&statuses);

        Ok(())
    }
}

fn summarize(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let success_create: Vec<_> = statuses.iter().filter(|s| s.is_success()).collect();
    let conflicts: Vec<_> = statuses
        .iter()
        .filter(|s| s.repo_status.is_conflict())
        .collect();
    let stashes: Vec<_> = statuses
        .iter()
        .filter(|s| s.stash_status.is_success())
        .collect();

    if !success_create.is_empty() {
        let msg = format!("\nSuccessfully pulled {} repos!\n", success_create.len());
        println!("{}", msg.green());
    }

    if !conflicts.is_empty() {
        let msg = format!(
            "There are {} repos has conflicts that need to resolve before pulling",
            conflicts.len()
        );
        println!("{}\n", msg.yellow());
    }

    if !stashes.is_empty() {
        let msg = format!("There are {} repos have been stashed that need to use \"stash apply\" to bring the changes back", stashes.len());
        println!("{}\n", msg.yellow());
    }

    if errors.is_empty() {
        println!("There is no error!\n");
    } else {
        let msg = format!("There {} errors when process command:", errors.len());
        println!("{}\n", msg.red());

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
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Pull Status", "Repo Status", "Stash Status"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn pull(dir: &PathBuf, user: &User, stash: bool) -> Status {
    let mut dir_name = "".to_string();
    let mut repo_status = RepoStatus::Clean;
    let mut stash_status = StashStatus::No;

    let mut pull = || -> Result<MergeStatus> {
        dir_name = path::dir_name(&dir)?;
        log::info!("Processing repo {}", dir_name);

        let mut git_repo =
            git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

        let status = git::status(&git_repo, false)?;

        if status.is_empty() {
            stash_status = StashStatus::No;
            repo_status = RepoStatus::Clean;
            // pull
            let cred = GitCredential::from(user);
            let status = git::pull(&git_repo, "origin", Some(cred))?;
            Ok(status)
        } else {
            if status.conflicted.is_empty() {
                repo_status = RepoStatus::Dirty;

                if stash {
                    // do stash
                    stash_status = match git::stash(&mut git_repo, None) {
                        Ok(_) => StashStatus::Success,
                        Err(e) => StashStatus::Failed(e),
                    };
                    // pull
                    let cred = GitCredential::from(user);
                    let status = git::pull(&git_repo, "origin", Some(cred))?;
                    return Ok(status);
                }
            } else {
                repo_status = RepoStatus::Conflict;
            }

            stash_status = StashStatus::Skip;
            Ok(MergeStatus::Nothing)
        }
    };

    let status = pull();

    Status {
        repo: dir_name,
        status,
        repo_status,
        stash_status,
    }
}

#[derive(Debug)]
struct Status {
    repo: String,
    status: Result<MergeStatus>,
    repo_status: RepoStatus,
    stash_status: StashStatus,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![
            cell!(b -> &self.repo),
            self.status_to_cell(),
            self.repo_status.to_cell(),
            self.stash_status.to_cell(),
        ])
    }

    fn status_to_cell(&self) -> Cell {
        match &self.status {
            Ok(s) => merge_status_to_cell(&s),
            Err(_) => cell!(Frr -> "Failed"),
        }
    }

    fn is_success(&self) -> bool {
        self.status.is_ok()
            && (self.stash_status.is_success() || matches!(self.stash_status, StashStatus::No))
    }

    //fn is_success_with_stash(&self) -> bool {
    //self.status.is_ok() && self.stash_status.is_success()
    //}

    //fn is_success_without_stash(&self) -> bool {
    //self.status.is_ok() && matches!(self.stash_status, StashStatus::No)
    //}

    fn has_error(&self) -> bool {
        self.status.is_err() || matches!(self.stash_status, StashStatus::Failed(_))
    }

    fn to_error_row(&self) -> Row {
        let e = if let StashStatus::Failed(e1) = &self.stash_status {
            e1
        } else if let Err(e2) = &self.status {
            e2
        } else {
            panic!("This should have an error here");
        };

        let msg = format!("{:?}", e);
        let lines = common::sub_strings(msg.as_str(), 80);
        let lines = lines.join("\n");
        row!(cell!(b -> &self.repo), cell!(Fr -> lines.as_str()))
    }
}

fn merge_status_to_cell(status: &MergeStatus) -> Cell {
    match &status {
        MergeStatus::FastForward => cell!(Fgr -> "FastForward Merged"),
        MergeStatus::NormalMerge => cell!(Fgr -> "Merged"),
        MergeStatus::MergeWithConflict => cell!(Frr -> "Merged with Conflict"),
        MergeStatus::SkipByConflict => cell!(r -> "Skip merge by conflict"),
        MergeStatus::Nothing => cell!(r -> "-"),
    }
}

#[derive(Debug)]
enum StashStatus {
    No,
    Skip,
    Success,
    Failed(Error),
}

impl StashStatus {
    fn to_cell(&self) -> Cell {
        match &self {
            StashStatus::No => cell!(r -> "-"),
            StashStatus::Skip => cell!(r -> "-"),
            StashStatus::Success => cell!(Fgr -> "Success"),
            StashStatus::Failed(_) => cell!(Frr -> "Failed"),
        }
    }

    fn is_success(&self) -> bool {
        matches!(self, StashStatus::Success)
    }
}

#[derive(Debug)]
enum RepoStatus {
    Clean,
    Dirty,
    Conflict,
}

impl RepoStatus {
    fn to_cell(&self) -> Cell {
        match &self {
            RepoStatus::Clean => cell!(Fgr -> "Clean"),
            RepoStatus::Dirty => cell!(r -> "Dirty"),
            RepoStatus::Conflict => cell!(Frr -> "Conflict"),
        }
    }

    fn is_conflict(&self) -> bool {
        matches!(self, RepoStatus::Conflict)
    }
}
