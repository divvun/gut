use super::common::{self, OrgSummary};
use super::fetch::FetchArgs;
use crate::cli::OutputFormat;
use crate::filter::Filter;
use crate::git;
use crate::git::GitStatus;
use crate::path::dir_name;
use anyhow::{Context, Result};
use clap::Parser;
use prettytable::{Row, Table, format, row, cell};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Parser)]
/// Show git status of all repositories that match a pattern
pub struct StatusArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
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
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
    #[arg(long, short)]
    /// Fetch from remotes before showing status
    pub fetch: bool,
}

impl StatusArgs {
    pub fn run(&self, format: Option<OutputFormat>) -> Result<()> {
        let format = format.unwrap_or(OutputFormat::Table);

        if self.fetch {
            let fetch_args = FetchArgs {
                organisation: self.organisation.clone(),
                regex: self.regex.clone(),
                all_orgs: self.all_orgs,
            };
            fetch_args.run()?;
            println!();
        }

        common::run_for_orgs_with_summary(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_single_org(format, org),
            print_org_summary,
        )
    }

    fn run_single_org(&self, format: OutputFormat, organisation: &str) -> Result<OrgSummary> {
        let root = common::root()?;

        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        let statuses: Vec<_> = sub_dirs.iter().map(status).collect();

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
            }
        }

        Ok(OrgSummary {
            name: organisation.to_string(),
            total_repos: filtered_statuses.len(),
            unpushed_repo_count,
            uncommitted_repo_count,
            total_unadded,
            total_deleted,
            total_modified,
            total_conflicted,
            total_added,
        })
    }
}

fn status(dir: &PathBuf) -> StatusResult {
    let name = dir_name(dir).unwrap_or_else(|_| "Unknown".to_string());
    
    let result = (|| -> Result<RepoStatus> {
        let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

        let status = git::status(&git_repo, false)?;
        let branch = git::head_shorthand(&git_repo)?;
        let repo_status = RepoStatus {
            name: name.clone(),
            branch,
            status,
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
    let success_statuses: Vec<_> = statuses.iter().filter_map(|s| s.result.as_ref().ok()).collect();
    let errors: Vec<_> = statuses.iter().filter(|s| s.result.is_err()).collect();
    
    let rows = to_rows_with_errors_sorted(&success_statuses.iter().map(|&s| s.clone()).collect::<Vec<_>>(), &errors, verbose);
    let table = to_table(&rows);
    table.printstd();
    
    if !errors.is_empty() {
        println!("\nThere were errors processing {} repositories:\n", errors.len());
        let mut error_table = Table::new();
        error_table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        error_table.set_titles(row!["Repo", "Error"]);
        
        for status_result in errors {
            if let Err(error) = &status_result.result {
                let msg = format!("{:?}", error);
                let lines = common::sub_strings(msg.as_str(), 80);
                let lines = lines.join("\n");
                error_table.add_row(row![cell!(b -> &status_result.name), cell!(Fr -> lines.as_str())]);
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
        row!["Repo", "branch", r -> "±origin", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"],
    );
    table
}

#[allow(dead_code)]
fn to_rows(statuses: &[RepoStatus], verbose: bool) -> Vec<StatusRow> {
    let mut rows: Vec<_> = statuses.iter().flat_map(|s| s.to_rows(verbose)).collect();
    rows.append(&mut to_total_summarize(statuses));
    rows
}

fn to_rows_with_errors_sorted(statuses: &[RepoStatus], errors: &[&StatusResult], verbose: bool) -> Vec<StatusRow> {
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
    };
    rows.push(summarize_row);
    rows
}

#[derive(Debug, Clone, Serialize)]
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
        let mut rows = vec![self.to_repo_summarize()];
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
                row![name, "-", r -> "-", r -> "-", r -> "-", r -> "-", r -> "-", r -> "-"]
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
            } => {
                row![total, uncommitted_repo_count, r -> unpushed_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added]
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
            } => {
                row![org_name, total_repos, r -> unpushed_repo_count, r -> uncommitted_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added]
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
        row!["Organisation", "#repos", r -> "±origin", r -> "Dirty", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"],
    );
    table
}
