use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::GitStatus;
use crate::path::dir_name;
use anyhow::{Context, Result};
use prettytable::{cell, format, row, Row, Table};
use rayon::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Show git status of all repositories that match a pattern
pub struct StatusArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// Option to show more detail
    pub verbose: bool,
}

impl StatusArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        let statuses: Vec<_> = sub_dirs.par_iter().map(|d| status(&d)).collect();
        let statuses: Result<Vec<_>> = statuses.into_par_iter().collect();
        let statuses: Vec<_> = statuses?;

        let rows = to_rows(&statuses, self.verbose);
        let table = to_table(&rows);

        table.printstd();
        Ok(())
    }
}

fn status(dir: &PathBuf) -> Result<RepoStatus> {
    let name = dir_name(dir)?;
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let status = git::status(&git_repo, false)?;
    let branch = git::head_shorthand(&git_repo)?;
    let repo_status = RepoStatus {
        name,
        branch,
        status,
    };
    Ok(repo_status)
}

fn to_table(statuses: &[StatusRow]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(
        row!["Repo", "branch", r -> "±origin", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"],
    );
    table
}

fn to_rows(statuses: &[RepoStatus], verbose: bool) -> Vec<StatusRow> {
    let mut rows: Vec<_> = statuses
        .iter()
        .map(|s| s.to_rows(verbose))
        .flatten()
        .collect();
    rows.append(&mut to_total_summarize(statuses));
    rows
}

fn to_total_summarize(statuses: &[RepoStatus]) -> Vec<StatusRow> {
    let mut rows = vec![StatusRow::TitleSeperation, StatusRow::SummarizeTitle];
    let total = statuses.len().to_string();
    let mut unpushed_repo_count: usize = 0;
    let mut uncommited_repo_count: usize = 0;
    let mut total_unadded: usize = 0;
    let mut total_deleted: usize = 0;
    let mut total_modified: usize = 0;
    let mut total_conflicted: usize = 0;
    let mut total_added: usize = 0;

    for status in statuses {
        if !status.status.is_empty() {
            uncommited_repo_count += 1;
        }
        if status.status.is_ahead > 0 || status.status.is_behind > 0 {
            unpushed_repo_count += 1;
        }
        total_added += status.status.added.len();
        total_conflicted += status.status.conflicted.len();
        total_modified += status.status.modified.len();
        total_unadded += status.status.new.len();
        total_deleted += status.status.deleted.len();
    }

    let summarize_row = StatusRow::SummarizeAll {
        total,
        unpushed_repo_count: unpushed_repo_count.to_string(),
        uncommited_repo_count: uncommited_repo_count.to_string(),
        total_unadded: total_unadded.to_string(),
        total_deleted: total_deleted.to_string(),
        total_modified: total_modified.to_string(),
        total_conflicted: total_conflicted.to_string(),
        total_added: total_added.to_string(),
    };
    rows.push(summarize_row);
    rows
}

#[derive(Debug)]
struct RepoStatus {
    name: String,
    branch: String,
    status: GitStatus,
}

impl RepoStatus {
    fn to_rows(&self, verbose: bool) -> Vec<StatusRow> {
        if verbose {
            self.to_repo_detail()
        } else {
            vec![self.to_repo_summarize()]
        }
    }

    fn to_repo_detail(&self) -> Vec<StatusRow> {
        let mut rows = vec![];
        rows.push(self.to_repo_summarize());
        rows.append(&mut show_detail_changes("C", &self.status.conflicted));
        rows.append(&mut show_detail_changes("U", &self.status.new));
        rows.append(&mut show_detail_changes("D", &self.status.deleted));
        rows.append(&mut show_detail_changes("M", &self.status.modified));
        rows.append(&mut show_detail_changes("A", &self.status.added));
        rows.push(StatusRow::RepoSeperation);
        rows
    }

    fn to_repo_summarize(&self) -> StatusRow {
        StatusRow::RepoSummarize {
            name: self.name.to_string(),
            branch: self.branch.to_string(),
            ahead_behind: self.status.ahead_behind(),
            unadded: self.status.new.len().to_string(),
            deleted: self.status.deleted.len().to_string(),
            modified: self.status.modified.len().to_string(),
            conflicted: self.status.conflicted.len().to_string(),
            added: self.status.added.len().to_string(),
        }
    }
}

fn show_detail_changes(msg: &str, list: &[String]) -> Vec<StatusRow> {
    let mut rows = vec![];
    if !list.is_empty() {
        for l in list {
            let fs = StatusRow::FileDetail {
                status: msg.to_string(),
                path: l.clone(),
            };
            rows.push(fs);
        }
    }
    rows
}

#[derive(Debug)]
enum StatusRow {
    RepoSummarize {
        name: String,
        branch: String,
        ahead_behind: String,
        unadded: String,
        deleted: String,
        modified: String,
        conflicted: String,
        added: String,
    },
    FileDetail {
        status: String,
        path: String,
    },
    SummarizeAll {
        total: String,
        unpushed_repo_count: String,
        uncommited_repo_count: String,
        total_unadded: String,
        total_deleted: String,
        total_modified: String,
        total_conflicted: String,
        total_added: String,
    },
    RepoSeperation,
    TitleSeperation,
    SummarizeTitle,
}

impl StatusRow {
    fn to_row(&self) -> Row {
        match self {
            StatusRow::RepoSeperation => row!["--------------"],
            StatusRow::TitleSeperation => row!["================"],
            StatusRow::FileDetail { status, path } => row![r => status, path],
            StatusRow::SummarizeAll {
                total,
                unpushed_repo_count,
                uncommited_repo_count,
                total_unadded,
                total_deleted,
                total_modified,
                total_conflicted,
                total_added,
            } => {
                row![total, uncommited_repo_count, r -> unpushed_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added]
            }
            StatusRow::RepoSummarize {
                name,
                branch,
                ahead_behind,
                unadded,
                deleted,
                modified,
                conflicted,
                added,
            } => {
                row![name, branch, r -> ahead_behind, r -> unadded, r -> deleted, r -> modified, r -> conflicted, r -> added]
            }
            StatusRow::SummarizeTitle => {
                row!["Repo Count", "Dirty", "fetch/push", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"]
            }
        }
    }
}
