use super::common;
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use prettytable::{Table, format, row};

#[derive(Debug, Parser)]
/// Do git clean -f for all local repositories that match a pattern
pub struct CleanArgs {
    #[arg(long, short)]
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
                    },
                    Err(e) => {
                        println!("Failed to process organization '{}': {:?}", org, e);
                    }
                }
            }
            
            print_clean_summary(&summaries);
            
            Ok(())
        } else {
            let organisation = common::organisation(self.organisation.as_deref())?;
            self.run_for_organization(&organisation, common_args)?;
            Ok(())
        }
    }

    fn run_for_organization(&self, organisation: &str, _common_args: &CommonArgs) -> Result<common::OrgResult> {
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
        println!("Files/directories get removed: ");
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

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
    }

    println!("\n=== All org summary ===");
    table.printstd();
}
