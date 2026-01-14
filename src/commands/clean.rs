use super::common;
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use prettytable::{Table, format, row};
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Do git clean -f for all local repositories that match a pattern
pub struct CleanArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl CleanArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        common::run_for_orgs_or_single(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org, common_args),
            Some(print_clean_summary),
        )
    }

    fn run_for_organization(
        &self,
        organisation: &str,
        _common_args: &CommonArgs,
    ) -> Result<common::OrgResult> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        let total_count = sub_dirs.len();
        let mut success_count = 0;
        let mut fail_count = 0;

        for dir in sub_dirs {
            match clean(&dir) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    fail_count += 1;
                    println!("Failed to clean dir {:?} because {:?}", dir, e);
                }
            }
        }

        Ok(common::OrgResult {
            org_name: organisation.to_string(),
            total_repos: total_count,
            successful_repos: success_count,
            failed_repos: fail_count,
            dirty_repos: 0,
        })
    }
}

fn clean(dir: &PathBuf) -> Result<()> {
    println!("Cleaning {:?}", dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let status = git::status(&git_repo, false)?;
    //println!("git status {:?}", status);

    if status.new.is_empty() {
        println!("Nothing to clean!\n");
    } else {
        println!("Files/directories removed: ");
        for f in status.new {
            let rf = dir.join(f);
            path::remove_path(&rf).with_context(|| format!("Cannot remove {:?}", rf))?;
            println!("{:?}", rf);
        }
        println!();
    }

    Ok(())
}

fn print_clean_summary(summaries: &[common::OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Cleaned", "Failed"]);

    let mut total_repos = 0;
    let mut total_cleaned = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_cleaned += summary.successful_repos;
        total_failed += summary.failed_repos;
    }

    table.add_empty_row();

    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_cleaned,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
