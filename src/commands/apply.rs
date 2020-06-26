use super::common;
use super::models::Script;
use crate::filter::Filter;
use crate::path;
use anyhow::{Error, Result};
use colored::*;
use prettytable::{cell, format, row, Cell, Row, Table};
use rayon::prelude::*;
use std::path::PathBuf;
use std::process::Output;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Apply a script to all local repositories that match a pattern
pub struct ApplyArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// The location of a script
    pub script: Script,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let sub_dirs = common::read_dirs_for_org(&organisation, &root, self.regex.as_ref())?;

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} that matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        let statuses: Vec<_> = sub_dirs
            .par_iter()
            .map(|r| apply_script(&r, script_path))
            .collect();

        summarize(&statuses);

        Ok(())
    }
}

fn apply_script(dir: &PathBuf, script: &str) -> Status {
    let mut dir_name = "".to_string();
    let mut apply = || -> Result<Output> {
        dir_name = path::dir_name(&dir)?;
        common::apply_script(dir, script)
    };
    let result = apply();

    Status {
        repo: dir_name,
        result,
    }
}

struct Status {
    repo: String,
    result: Result<Output, Error>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo), self.status(), self.output()])
    }

    fn status(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Success"),
            Err(_) => cell!(Frr -> "Failed"),
        }
    }

    fn output(&self) -> Cell {
        match &self.result {
            Ok(o) => {
                if o.status.success() {
                    let msg = str_from_v8(&o.stdout);
                    cell!(Fgl -> msg.as_str())
                } else {
                    let msg = str_from_v8(&o.stderr);
                    cell!(Frl -> msg.as_str())
                }
            }
            Err(_) => cell!(r -> "-"),
        }
    }

    fn has_error(&self) -> bool {
        matches!(self.result, Err(_))
    }

    fn to_error_row(&self) -> Row {
        let e = if let Err(e) = &self.result {
            e
        } else {
            panic!("This should have an error here");
        };

        let msg = format!("{:?}", e);
        let lines = common::sub_strings(msg.as_str(), 80);
        let lines = lines.join("\n");
        row!(cell!(b -> &self.repo), cell!(Fr -> lines.as_str()))
    }
}

fn str_from_v8(v8: &[u8]) -> String {
    match std::str::from_utf8(v8) {
        Ok(s) => s.to_string(),
        Err(_) => "-".to_string(),
    }
    //let lines = common::sub_strings(&msg, 80);
    //lines.join("\n")
}

fn to_table(statuses: &[Status]) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status", "Output"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn summarize(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!(
            "\nApplied the script for {:?} repos successfully",
            successes.len()
        );
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!(
            "There are {} errors when executing the command:",
            errors.len()
        );
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
