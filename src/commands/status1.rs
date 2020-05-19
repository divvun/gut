use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::GitStatus;
use crate::path::dir_name;
use anyhow::{Context, Result};
use prettytable::{cell, format, row, Row, Table};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct StatusArgs1 {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub verbose: bool,
}

impl StatusArgs1 {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        println!("Start {:?}", sub_dirs);
        let s: Vec<_> = sub_dirs.iter()
            .map(|d| status(&d))
            .collect();
        println!("Start {:?}", s);
        let s: Result<Vec<_>> = s.into_iter().collect();
        println!("Start {:?}", s);
        let s: Vec<_> = s?.iter()
            .map(|s| to_repo_summarize(&s))
            .collect();

        println!("Summarize {:?}", s);
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

fn process(statuses: &[RepoStatus]) -> Vec<StatusRow> {
    statuses.iter().map(|s| to_repo_summarize(&s)).collect()
}

fn to_repo_summarize(status: &RepoStatus) -> StatusRow {
    StatusRow::RepoSummarize {
        name: status.name.to_string(),
        branch: status.name.to_string(),
        ahead_behind: status.status.ahead_behind(),
        unadded: status.status.new.len().to_string(),
        deleted: status.status.deleted.len().to_string(),
        modified: status.status.modified.len().to_string(),
        conflicted: status.status.conflicted.len().to_string(),
        added: status.status.added.len().to_string(),
    }
}

#[derive(Debug)]
struct RepoStatus {
    name: String,
    branch: String,
    status: GitStatus,
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
        total_deleteed: String,
        total_modified: String,
        total_conflicted: String,
        total_added: String
    }
}

//static StatusTitle: Vec<&str> = vec!["Repo", "branch", "±origin", "U D M C A"];
