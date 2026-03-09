use super::common::{self, OrgSummary};
use super::fetch::FetchArgs;
use crate::cli::OutputFormat;
use crate::filter::Filter;
use crate::git;
use crate::git::GitStatus;
use crate::git::lfs::{self, LfsFileStatus};
use crate::path::dir_name;
use crate::system_health;
use anyhow::{Context, Result};
use clap::Parser;
use prettytable::{Row, Table, cell, format, row};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Parser)]
/// Show git status of all repositories that match a pattern
pub struct StatusArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Option to show more detail
    pub verbose: bool,
    #[arg(long, short)]
    /// Option to omit repositories without changes
    pub quiet: bool,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
    #[arg(long, short)]
    /// Fetch from remotes before showing status
    pub fetch: bool,
    #[arg(long, short)]
    /// Show detailed LFS file download status
    pub lfs: bool,
}

impl StatusArgs {
    pub fn run(&self, format: Option<OutputFormat>) -> Result<()> {
        let format = format.unwrap_or(OutputFormat::Table);

        if self.fetch {
            let fetch_args = FetchArgs {
                owner: self.owner.clone(),
                regex: self.regex.clone(),
                all_owners: self.all_owners,
            };
            fetch_args.run()?;
            println!();
        }

        common::run_for_owners_with_summary(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(format, owner),
            print_org_summary,
        )
    }

    fn run_for_owner(&self, format: OutputFormat, owner: &str) -> Result<OrgSummary> {
        let root = common::root()?;

        let sub_dirs = common::read_dirs_for_owner(owner, &root, self.regex.as_ref())?;

        let detailed_lfs = self.lfs;
        let statuses = common::process_with_progress(
            "Status",
            &sub_dirs,
            |dir| status(dir, detailed_lfs),
            |result| result.name.clone(),
        );

        let filtered_statuses: Vec<_> = statuses
            .iter()
            .filter(|status_result| {
                if let Ok(status) = &status_result.result {
                    !(self.quiet
                        && status.status.is_empty()
                        && status.status.is_ahead == 0
                        && status.status.is_behind == 0)
                } else {
                    true // Always show errors
                }
            })
            .cloned()
            .collect();

        match format {
            OutputFormat::Json => {
                let json_statuses: Vec<_> = filtered_statuses
                    .iter()
                    .filter_map(|s| s.result.as_ref().ok())
                    .collect();
                println!("{}", json!(json_statuses));
            }
            OutputFormat::Table => {
                print_status_table(&filtered_statuses, self.verbose);
            }
        }

        // Lag organizasjon-sammandrag med same statistikk som summarize
        let mut unpushed_repo_count = 0;
        let mut uncommitted_repo_count = 0;
        let mut total_unadded = 0;
        let mut total_deleted = 0;
        let mut total_modified = 0;
        let mut total_conflicted = 0;
        let mut total_added = 0;
        let mut total_lfs_repos = 0;

        for status_result in &filtered_statuses {
            if let Ok(status) = &status_result.result {
                if !status.status.is_empty() {
                    uncommitted_repo_count += 1;
                }
                if status.status.is_ahead > 0 || status.status.is_behind > 0 {
                    unpushed_repo_count += 1;
                }
                total_added += status.status.added.len();
                total_conflicted += status.status.conflicted.len();
                total_modified += status.status.modified.len();
                total_unadded += status.status.new.len();
                total_deleted += status.status.deleted.len();
                if status.uses_lfs {
                    total_lfs_repos += 1;
                }
            }
        }

        Ok(OrgSummary {
            name: owner.to_string(),
            total_repos: filtered_statuses.len(),
            unpushed_repo_count,
            uncommitted_repo_count,
            total_unadded,
            total_deleted,
            total_modified,
            total_conflicted,
            total_added,
            total_lfs_repos,
        })
    }
}

fn status(dir: &PathBuf, detailed_lfs: bool) -> StatusResult {
    let name = dir_name(dir).unwrap_or_else(|_| "Unknown".to_string());

    let result = (|| -> Result<RepoStatus> {
        let git_repo =
            git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

        let status = git::status(&git_repo, false)?;
        let branch = git::head_shorthand(&git_repo)?;

        let uses_lfs = lfs::repo_uses_lfs(dir);
        let lfs_files = if detailed_lfs && uses_lfs && system_health::is_git_lfs_installed() {
            lfs::lfs_file_status(dir)
        } else {
            None
        };

        let repo_status = RepoStatus {
            name: name.clone(),
            branch,
            status,
            uses_lfs,
            lfs_files,
        };
        Ok(repo_status)
    })();

    StatusResult {
        name,
        result: result.map_err(Arc::new),
    }
}

#[derive(Debug, Clone)]
struct StatusResult {
    name: String,
    result: Result<RepoStatus, Arc<anyhow::Error>>,
}

fn print_status_table(statuses: &[StatusResult], verbose: bool) {
    let success_statuses: Vec<_> = statuses
        .iter()
        .filter_map(|s| s.result.as_ref().ok())
        .collect();
    let errors: Vec<_> = statuses.iter().filter(|s| s.result.is_err()).collect();

    let rows = to_rows_with_errors_sorted(
        &success_statuses
            .iter()
            .map(|&s| s.clone())
            .collect::<Vec<_>>(),
        &errors,
        verbose,
    );
    let table = to_table(&rows);
    table.printstd();

    if !errors.is_empty() {
        println!(
            "\nThere were errors processing {} repositories:\n",
            errors.len()
        );
        let mut error_table = Table::new();
        error_table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        error_table.set_titles(row!["Repo", "Error"]);

        for status_result in errors {
            if let Err(error) = &status_result.result {
                let msg = format!("{:?}", error);
                let lines = common::sub_strings(msg.as_str(), 80);
                let lines = lines.join("\n");
                error_table.add_row(row![
                    cell!(b -> &status_result.name),
                    cell!(Fr -> lines.as_str())
                ]);
            }
        }
        error_table.printstd();
    }
}

fn to_table(statuses: &[StatusRow]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(
        row!["Repo", "branch", r -> "±origin", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A", r -> "LFS"],
    );
    table
}

#[allow(dead_code)]
fn to_rows(statuses: &[RepoStatus], verbose: bool) -> Vec<StatusRow> {
    let mut rows: Vec<_> = statuses.iter().flat_map(|s| s.to_rows(verbose)).collect();
    rows.append(&mut to_total_summarize(statuses));
    rows
}

fn to_rows_with_errors_sorted(
    statuses: &[RepoStatus],
    errors: &[&StatusResult],
    verbose: bool,
) -> Vec<StatusRow> {
    // Create a sorted list of repo names with their types
    let mut all_repos: Vec<(&str, bool)> = Vec::new();

    // Add success repos
    for status in statuses {
        all_repos.push((&status.name, true));
    }

    // Add error repos
    for error_status in errors {
        all_repos.push((&error_status.name, false));
    }

    // Sort alphabetically by name
    all_repos.sort_by_key(|(name, _)| *name);

    // Create rows in sorted order
    let mut rows = Vec::new();
    for (name, is_success) in all_repos {
        if is_success {
            // Find the RepoStatus and add its rows
            if let Some(status) = statuses.iter().find(|s| s.name == name) {
                rows.extend(status.to_rows(verbose));
            }
        } else {
            // Add error row
            rows.push(StatusRow::ErrorRow {
                name: name.to_string(),
            });
        }
    }

    // Add summary at the end
    rows.append(&mut to_total_summarize(statuses));
    rows
}

fn to_total_summarize(statuses: &[RepoStatus]) -> Vec<StatusRow> {
    let mut rows = vec![StatusRow::TitleSeperation, StatusRow::SummarizeTitle];
    let total = statuses.len().to_string();
    let mut unpushed_repo_count: usize = 0;
    let mut uncommitted_repo_count: usize = 0;
    let mut total_unadded: usize = 0;
    let mut total_deleted: usize = 0;
    let mut total_modified: usize = 0;
    let mut total_conflicted: usize = 0;
    let mut total_added: usize = 0;
    let mut total_lfs_repos: usize = 0;

    for status in statuses {
        if !status.status.is_empty() {
            uncommitted_repo_count += 1;
        }
        if status.status.is_ahead > 0 || status.status.is_behind > 0 {
            unpushed_repo_count += 1;
        }
        total_added += status.status.added.len();
        total_conflicted += status.status.conflicted.len();
        total_modified += status.status.modified.len();
        total_unadded += status.status.new.len();
        total_deleted += status.status.deleted.len();
        if status.uses_lfs {
            total_lfs_repos += 1;
        }
    }

    let summarize_row = StatusRow::SummarizeAll {
        total,
        unpushed_repo_count: unpushed_repo_count.to_string(),
        uncommitted_repo_count: uncommitted_repo_count.to_string(),
        total_unadded: total_unadded.to_string(),
        total_deleted: total_deleted.to_string(),
        total_modified: total_modified.to_string(),
        total_conflicted: total_conflicted.to_string(),
        total_added: total_added.to_string(),
        total_lfs_repos: total_lfs_repos.to_string(),
    };
    rows.push(summarize_row);
    rows
}

#[derive(Debug, Clone, Serialize)]
struct RepoStatus {
    name: String,
    branch: String,
    status: GitStatus,
    uses_lfs: bool,
    lfs_files: Option<LfsFileStatus>,
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
        let mut rows = vec![self.to_repo_summarize()];
        rows.append(&mut show_detail_changes("C", &self.status.conflicted));
        rows.append(&mut show_detail_changes("U", &self.status.new));
        rows.append(&mut show_detail_changes("D", &self.status.deleted));
        rows.append(&mut show_detail_changes("M", &self.status.modified));
        rows.append(&mut show_detail_changes("A", &self.status.added));

        if let Some(ref lfs_files) = self.lfs_files {
            for file in &lfs_files.files {
                let indicator = if file.downloaded { "LFS*" } else { "LFS-" };
                rows.push(StatusRow::FileDetail {
                    status: indicator.to_string(),
                    path: file.name.clone(),
                });
            }
        }

        rows.push(StatusRow::RepoSeperation);
        rows
    }

    fn to_repo_summarize(&self) -> StatusRow {
        let lfs = if !self.uses_lfs {
            "-".to_string()
        } else if let Some(ref lfs_files) = self.lfs_files {
            // A locally modified LFS file is reported as "-" (pointer) by
            // `git lfs ls-files` because the working tree content no longer
            // matches the stored LFS object. Don't count these as undownloaded.
            let modified_not_downloaded = lfs_files
                .files
                .iter()
                .filter(|f| !f.downloaded && self.status.modified.iter().any(|m| m == &f.name))
                .count();
            let adjusted_downloaded = lfs_files.downloaded + modified_not_downloaded;
            if adjusted_downloaded >= lfs_files.total {
                "YES".to_string()
            } else {
                format!("{}/{}", adjusted_downloaded, lfs_files.total)
            }
        } else {
            "YES".to_string()
        };

        StatusRow::RepoSummarize {
            name: self.name.to_string(),
            branch: self.branch.to_string(),
            ahead_behind: self.status.ahead_behind(),
            unadded: self.status.new.len().to_string(),
            deleted: self.status.deleted.len().to_string(),
            modified: self.status.modified.len().to_string(),
            conflicted: self.status.conflicted.len().to_string(),
            added: self.status.added.len().to_string(),
            lfs,
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

#[derive(Debug, Clone)]
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
        lfs: String,
    },
    FileDetail {
        status: String,
        path: String,
    },
    SummarizeAll {
        total: String,
        unpushed_repo_count: String,
        uncommitted_repo_count: String,
        total_unadded: String,
        total_deleted: String,
        total_modified: String,
        total_conflicted: String,
        total_added: String,
        total_lfs_repos: String,
    },
    OrgSummarize {
        org_name: String,
        total_repos: String,
        unpushed_repo_count: String,
        uncommitted_repo_count: String,
        total_unadded: String,
        total_deleted: String,
        total_modified: String,
        total_conflicted: String,
        total_added: String,
        total_lfs_repos: String,
    },
    RepoSeperation,
    TitleSeperation,
    SummarizeTitle,
    Empty,
    ErrorRow {
        name: String,
    },
}

impl StatusRow {
    fn to_row(&self) -> Row {
        match self {
            StatusRow::RepoSeperation => row!["--------------"],
            StatusRow::TitleSeperation => row!["================"],
            StatusRow::Empty => row![""],
            StatusRow::ErrorRow { name } => {
                row![name, "-", r -> "-", r -> "-", r -> "-", r -> "-", r -> "-", r -> "-", r -> "-"]
            }
            StatusRow::FileDetail { status, path } => row![r => status, path],
            StatusRow::SummarizeAll {
                total,
                unpushed_repo_count,
                uncommitted_repo_count,
                total_unadded,
                total_deleted,
                total_modified,
                total_conflicted,
                total_added,
                total_lfs_repos,
            } => {
                row![total, uncommitted_repo_count, r -> unpushed_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added, r -> total_lfs_repos]
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
                lfs,
            } => {
                row![name, branch, r -> ahead_behind, r -> unadded, r -> deleted, r -> modified, r -> conflicted, r -> added, r -> lfs]
            }
            StatusRow::SummarizeTitle => {
                row!["Repo Count", "Dirty", "fetch/push", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A", r -> "LFS"]
            }
            StatusRow::OrgSummarize {
                org_name,
                total_repos,
                unpushed_repo_count,
                uncommitted_repo_count,
                total_unadded,
                total_deleted,
                total_modified,
                total_conflicted,
                total_added,
                total_lfs_repos,
            } => {
                row![org_name, total_repos, r -> unpushed_repo_count, r -> uncommitted_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added, r -> total_lfs_repos]
            }
        }
    }
}

pub fn print_org_summary(summaries: &[OrgSummary]) {
    let mut rows = vec![];

    let mut total_repos = 0;
    let mut total_unpushed = 0;
    let mut total_uncommited = 0;
    let mut total_unadded = 0;
    let mut total_deleted = 0;
    let mut total_modified = 0;
    let mut total_conflicted = 0;
    let mut total_added = 0;
    let mut total_lfs_repos = 0;

    for summary in summaries {
        let org_row = StatusRow::OrgSummarize {
            org_name: summary.name.clone(),
            total_repos: summary.total_repos.to_string(),
            unpushed_repo_count: summary.unpushed_repo_count.to_string(),
            uncommitted_repo_count: summary.uncommitted_repo_count.to_string(),
            total_unadded: summary.total_unadded.to_string(),
            total_deleted: summary.total_deleted.to_string(),
            total_modified: summary.total_modified.to_string(),
            total_conflicted: summary.total_conflicted.to_string(),
            total_added: summary.total_added.to_string(),
            total_lfs_repos: summary.total_lfs_repos.to_string(),
        };
        rows.push(org_row);

        total_repos += summary.total_repos;
        total_unpushed += summary.unpushed_repo_count;
        total_uncommited += summary.uncommitted_repo_count;
        total_unadded += summary.total_unadded;
        total_deleted += summary.total_deleted;
        total_modified += summary.total_modified;
        total_conflicted += summary.total_conflicted;
        total_added += summary.total_added;
        total_lfs_repos += summary.total_lfs_repos;
    }

    // Add separator row
    rows.push(StatusRow::Empty);

    // Add total row
    let total_row = StatusRow::OrgSummarize {
        org_name: "TOTAL".to_string(),
        total_repos: total_repos.to_string(),
        unpushed_repo_count: total_unpushed.to_string(),
        uncommitted_repo_count: total_uncommited.to_string(),
        total_unadded: total_unadded.to_string(),
        total_deleted: total_deleted.to_string(),
        total_modified: total_modified.to_string(),
        total_conflicted: total_conflicted.to_string(),
        total_added: total_added.to_string(),
        total_lfs_repos: total_lfs_repos.to_string(),
    };
    rows.push(total_row);

    let table = to_org_summary_table(&rows);
    println!("\n=== All org summary ===");
    table.printstd();
}

fn to_org_summary_table(statuses: &[StatusRow]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(
        row!["Owner", "#repos", r -> "±origin", r -> "Dirty", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A", r -> "LFS"],
    );
    table
}
