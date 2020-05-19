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

        Ok(())
    }
}

struct RepoStatus {
    name: String,
    branch: String,
    status: GitStatus,
}

enum StatusRow {
    Summarize {
        name: String,
        branch: String,
        ahead_behind: String,
        un_added: String,
        deleted: String,
        modified: String,
        conflicted: String,
        added: String,
    },
    FileDetail {
        status: String,
        path: String,
    },
}

static StatusTitle: &Vec<&str> = vec!["Repo", "branch", "±origin", "U D M C A"];

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
    Ok()
}
