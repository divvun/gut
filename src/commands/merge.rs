use super::common::{self, OrgResult};
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::git;
use crate::git::MergeStatus;
use anyhow::{Context, Result};
use clap::Parser;
use prettytable::{Table, format, row};
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Merge a branch to the current branch for all repositories that match a pattern
pub struct MergeArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// The branch to be merged
    pub branch: String,
    #[arg(long, short)]
    /// Option to abort merging process if there is a conflict
    pub abort_if_conflict: bool,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl MergeArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        common::run_for_orgs(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org, common_args),
            Some(print_merge_summary),
        )
    }

    fn run_for_organization(
        &self,
        organisation: &str,
        _common_args: &CommonArgs,
    ) -> Result<OrgResult> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        let total_count = sub_dirs.len();
        let mut success_count = 0;
        let mut fail_count = 0;

        for dir in sub_dirs {
            match merge(&dir, &self.branch, self.abort_if_conflict) {
                Ok(status) => {
                    success_count += 1;
                    match status {
                        MergeStatus::FastForward => println!("Merge fast forward"),
                        MergeStatus::NormalMerge => {
                            println!("Merge made by the 'recursive' strategy")
                        }
                        MergeStatus::MergeWithConflict => {
                            println!(
                                "Auto merge failed. Fix conflicts and then commit the results."
                            )
                        }
                        MergeStatus::Nothing => println!("Already up to date"),
                        MergeStatus::SkipByConflict => {
                            println!("There are conflict(s), and we skipped")
                        }
                    }
                }
                Err(e) => {
                    fail_count += 1;
                    println!(
                        "Failed to merge branch {} for dir {:?} because {:?}",
                        self.branch, dir, e
                    );
                }
            }
        }

        Ok(OrgResult {
            org_name: organisation.to_string(),
            total_repos: total_count,
            successful_repos: success_count,
            failed_repos: fail_count,
            dirty_repos: 0,
        })
    }
}

fn merge(dir: &PathBuf, target: &str, abort: bool) -> Result<git::MergeStatus> {
    println!("Merging branch {} into head for {:?}", target, dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let merge_status = git::merge_local(&git_repo, target, abort)?;
    Ok(merge_status)
}

fn print_merge_summary(summaries: &[OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Merged", "Failed"]);

    let mut total_repos = 0;
    let mut total_merged = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_merged += summary.successful_repos;
        total_failed += summary.failed_repos;
    }

    table.add_empty_row();

    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_merged,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
