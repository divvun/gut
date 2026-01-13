use super::common;
use super::models::Script;
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::path;
use anyhow::{Error, Result};
use clap::Parser;
use colored::*;
use prettytable::{Cell, Row, Table, cell, format, row};
use rayon::prelude::*;
use std::env;
use std::path::PathBuf;
use std::process::Output;

#[derive(Debug, Parser)]
/// Apply a script to all local repositories that match a pattern
///
/// If you want your script to use your authentication token, you
/// can refer to it in your script with $GUT_TOKEN
pub struct ApplyArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// The location of a script
    pub script: Script,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl ApplyArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        if self.all_orgs {
            let organizations = common::get_all_organizations()?;
            if organizations.is_empty() {
                println!("No organizations found in root directory");
                return Ok(());
            }

            let mut summaries = Vec::new();

            for org in &organizations {
                println!("\n=== Processing organization: {} ===", org);

                match self.run_for_organization(org, common_args) {
                    Ok(summary) => {
                        summaries.push(summary);
                    }
                    Err(e) => {
                        println!("Failed to process organization '{}': {:?}", org, e);
                    }
                }
            }

            print_apply_summary(&summaries);

            Ok(())
        } else {
            let organisation = common::organisation(self.organisation.as_deref())?;
            self.run_for_organization(&organisation, common_args)?;
            Ok(())
        }
    }

    fn run_for_organization(
        &self,
        organisation: &str,
        _common_args: &CommonArgs,
    ) -> Result<common::OrgResult> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        // set auth_token to env
        let user_token = common::user_token()?;
        let key = "GUT_TOKEN";
        unsafe { env::set_var(key, user_token) };

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} that match the pattern {:?}",
                organisation, self.regex
            );
            return Ok(common::OrgResult {
                org_name: organisation.to_string(),
                total_repos: 0,
                successful_repos: 0,
                failed_repos: 0,
                dirty_repos: 0,
            });
        }

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        let statuses: Vec<_> = sub_dirs
            .par_iter()
            .map(|r| apply_script(r, script_path))
            .collect();

        summarize(&statuses);

        let total_count = statuses.len();
        let success_count = statuses.iter().filter(|s| !s.has_error()).count();
        let fail_count = statuses.iter().filter(|s| s.has_error()).count();

        Ok(common::OrgResult {
            org_name: organisation.to_string(),
            total_repos: total_count,
            successful_repos: success_count,
            failed_repos: fail_count,
            dirty_repos: 0,
        })
    }
}

fn apply_script(dir: &PathBuf, script: &str) -> Status {
    let mut dir_name = "".to_string();
    let mut apply = || -> Result<Output> {
        dir_name = path::dir_name(dir)?;
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
        self.result.is_err()
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

fn print_apply_summary(summaries: &[common::OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Applied", "Failed"]);

    let mut total_repos = 0;
    let mut total_applied = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_applied += summary.successful_repos;
        total_failed += summary.failed_repos;
    }

    table.add_empty_row();

    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_applied,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
