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
pub struct StatusArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub verbose: bool,
}

impl StatusArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        table.set_titles(row!["Repo", "branch", "±origin", "U D M C A"]);

        println!("Status for repos in org {}\n", self.organisation);

        let mut err_messages = vec![];
        let mut unpushed_repo_count: usize = 0;
        let mut uncommited_repo_count: usize = 0;

        for dir in sub_dirs {
            let result = status(&dir, self.verbose);
            match result {
                Ok((status, rows)) => {
                    for row in rows {
                        table.add_row(row);
                    }
                    if !status.is_empty() {
                        uncommited_repo_count += 1;
                    }
                    if status.is_ahead > 0 || status.is_behind > 0 {
                        unpushed_repo_count += 1;
                    }
                }
                Err(e) => err_messages.push(format!(
                    "Failed to status git status for dir {:?} because {:?}",
                    dir, e
                )),
            }
        }
        table.add_row(row!["====="]);
        table.add_row(row![
            "Total",
            "",
            unpushed_repo_count.to_string(),
            uncommited_repo_count.to_string()
        ]);
        table.printstd();
        Ok(())
    }
}

fn status(dir: &PathBuf, verbose: bool) -> Result<(GitStatus, Vec<Row>)> {
    let dir_name = dir_name(dir)?;
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let status = git::status(&git_repo, false)?;
    let current_branch = git::head_shorthand(&git_repo)?;

    let rows = if verbose {
        show_detail(&dir_name, &current_branch, &status)
    } else {
        change_summarize(&dir_name, &current_branch, &status)
    };

    Ok((status, rows))
}

fn change_summarize(dir_name: &str, branch: &str, status: &GitStatus) -> Vec<Row> {
    let ahead_behind = status.ahead_behind();

    // U D M C
    let change = format!(
        "{} {} {} {} {}",
        &status.new.len(),
        &status.deleted.len(),
        &status.modified.len(),
        &status.conflicted.len(),
        &status.added.len()
    );

    vec![row![dir_name, branch, ahead_behind, change]]
}

fn show_detail(dir_name: &str, branch: &str, status: &GitStatus) -> Vec<Row> {
    let mut rows = vec![];
    rows.push(change_summarize(dir_name, branch, status));
    rows.push(show_detail_changes("C", &status.conflicted));
    rows.push(show_detail_changes("U", &status.new));
    rows.push(show_detail_changes("D", &status.deleted));
    rows.push(show_detail_changes("M", &status.modified));
    rows.push(show_detail_changes("A", &status.added));
    if !status.is_empty() {
        rows.push(vec![row!["----"]]);
    }
    rows.concat()
}

fn show_detail_changes(msg: &str, list: &[String]) -> Vec<Row> {
    let mut rows = vec![];
    if !list.is_empty() {
        for l in list {
            let m = format!("{} {}", msg, l);
            rows.push(row![m]);
        }
    }
    rows
}
