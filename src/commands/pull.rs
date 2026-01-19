use super::common::{self, OrgResult};
use crate::cli::OutputFormat;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::git::PullStatus;
use crate::path;
use crate::user::User;
use anyhow::{Context, Error, Result};
use clap::Parser;
use colored::*;
use prettytable::{Cell, Row, Table, cell, format, row};
use rayon::prelude::*;
use serde::{Serialize, Serializer};
use serde_json::json;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Parser)]
/// Pull the current branch of all local repositories that match a regex
///
/// This command only works on those repositories that has been cloned in root directory
///
pub struct PullArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Option to stash if there are unstaged changes
    pub stash: bool,
    #[arg(long, short)]
    /// Option to create a merge commit instead of rebase
    pub merge: bool,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl PullArgs {
    pub fn run(&self, format: Option<OutputFormat>) -> Result<()> {
        let format = format.unwrap_or(OutputFormat::Table);
        common::run_for_orgs_with_summary(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(format, org),
            print_pull_summary,
        )
    }

    fn run_for_organization(&self, format: OutputFormat, organisation: &str) -> Result<OrgResult> {
        let user = common::user()?;
        let root = common::root()?;

        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} that match the pattern {:?}",
                organisation, self.regex
            );
            return Ok(OrgResult::new(organisation));
        }

        let statuses = common::process_with_progress(
            "Pulling",
            &sub_dirs,
            |d| pull(d, &user, self.stash, self.merge),
            |status| status.repo.clone(),
        );

        // Count successful vs failed pulls
        let successful_pulls = statuses.iter().filter(|s| s.was_updated()).count();
        let failed_pulls = statuses.iter().filter(|s| s.has_error()).count();
        let dirty_repos = statuses.iter().filter(|s| s.is_dirty()).count();

        match format {
            OutputFormat::Json => println!("{}", json!(statuses)),
            OutputFormat::Table => summarize(&statuses),
        };

        Ok(OrgResult {
            org_name: organisation.to_string(),
            total_repos: sub_dirs.len(),
            successful_repos: successful_pulls,
            failed_repos: failed_pulls,
            dirty_repos,
        })
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
        let msg = format!(
            "There are {} repos have been stashed that need to use \"stash apply\" to bring the changes back",
            stashes.len()
        );
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
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Pull Status", "Repo Status", "Stash Status"]);
    table
}

fn pull(dir: &PathBuf, user: &User, stash: bool, merge: bool) -> Status {
    let mut dir_name = "".to_string();
    let mut repo_status = RepoStatus::Clean;
    let mut stash_status = StashStatus::No;

    let mut pull = || -> Result<PullStatus> {
        dir_name = path::dir_name(dir)?;

        let mut git_repo =
            git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

        let status = git::status(&git_repo, false)?;

        if !status.is_dirty() {
            stash_status = StashStatus::No;
            repo_status = RepoStatus::Clean;
            // pull
            let cred = GitCredential::from(user);
            let status = git::pull(&git_repo, "origin", Some(cred), merge)?;
            Ok(status)
        } else {
            if status.conflicted.is_empty() {
                repo_status = RepoStatus::Dirty;

                if stash {
                    // do stash
                    stash_status = match git::stash(&mut git_repo, None) {
                        Ok(_) => StashStatus::Success,
                        Err(e) => StashStatus::Failed(Arc::new(e)),
                    };
                    // pull
                    let cred = GitCredential::from(user);
                    let status = git::pull(&git_repo, "origin", Some(cred), merge)?;
                    return Ok(status);
                }
            } else {
                repo_status = RepoStatus::Conflict;
            }

            stash_status = StashStatus::Skip;
            Ok(PullStatus::Nothing)
        }
    };

    let status = pull().map_err(Arc::new);

    Status {
        repo: dir_name,
        status,
        repo_status,
        stash_status,
    }
}

fn serialize_status<S>(
    status: &Result<PullStatus, Arc<anyhow::Error>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match status {
        Ok(pull_status) => pull_status.serialize(s),
        Err(e) => s.serialize_str(&e.to_string()),
    }
}

#[derive(Debug, Clone, Serialize)]
struct Status {
    repo: String,
    #[serde(serialize_with = "serialize_status")]
    status: Result<PullStatus, Arc<anyhow::Error>>,
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
            Ok(s) => merge_status_to_cell(s),
            Err(_) => cell!(Frr -> "Failed"),
        }
    }

    fn is_success(&self) -> bool {
        self.status.is_ok()
            && (self.stash_status.is_success() || matches!(self.stash_status, StashStatus::No))
    }

    fn was_updated(&self) -> bool {
        if let Ok(pull_status) = &self.status {
            !matches!(pull_status, PullStatus::Nothing)
        } else {
            false
        }
    }

    fn is_dirty(&self) -> bool {
        matches!(self.repo_status, RepoStatus::Dirty | RepoStatus::Conflict)
    }

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

fn merge_status_to_cell(status: &PullStatus) -> Cell {
    match &status {
        PullStatus::FastForward => cell!(Fgr -> "FastForward Merged"),
        PullStatus::Normal => cell!(Fgr -> "Pulled"),
        PullStatus::WithConflict => cell!(Frr -> "Pulled with Conflict"),
        PullStatus::SkipConflict => cell!(r -> "Skip pull by conflict"),
        PullStatus::Nothing => cell!(r -> "-"),
    }
}

fn serialize_error<S>(err: &Arc<Error>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{}", err))
}

#[derive(Debug, Clone, Serialize)]
enum StashStatus {
    No,
    Skip,
    Success,
    #[serde(serialize_with = "serialize_error")]
    Failed(Arc<Error>),
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

#[derive(Debug, Clone, Serialize)]
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

fn print_pull_summary(summaries: &[OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Updated", "Failed", "Dirty"]);

    let mut total_repos = 0;
    let mut total_updated = 0;
    let mut total_failed = 0;
    let mut total_dirty = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos,
            r -> summary.dirty_repos
        ]);
        total_repos += summary.total_repos;
        total_updated += summary.successful_repos;
        total_failed += summary.failed_repos;
        total_dirty += summary.dirty_repos;
    }

    table.add_empty_row();

    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_updated,
        r -> total_failed,
        r -> total_dirty
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
