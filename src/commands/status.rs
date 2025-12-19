use super::common;
use crate::cli::{Args as CommonArgs, OutputFormat};
use crate::filter::Filter;
use crate::git;
use crate::git::GitStatus;
use crate::path::dir_name;
use anyhow::{Context, Result};
use clap::Parser;
use prettytable::{Row, Table, format, row};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Show git status of all repositories that match a pattern
pub struct StatusArgs {
    #[arg(long, short)]
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
}

impl StatusArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        if self.all_orgs {
            let organizations = common::get_all_organizations()?;
            if organizations.is_empty() {
                println!("No organizations found in root directory");
                return Ok(());
            }
            
            let mut org_summaries = Vec::new();
            
            for org in &organizations {
                println!("\n=== Processing organization: {} ===", org);
                
                match self.run_single_org(common_args, org) {
                    Ok(summary) => {
                        org_summaries.push(summary);
                    }
                    Err(e) => {
                        println!("Failed to process organization '{}': {:?}", org, e);
                        // Create error entry with zero values
                        let error_summary = common::OrgSummary {
                            name: org.clone(),
                            total_repos: 0,
                            unpushed_repo_count: 0,
                            uncommited_repo_count: 0,
                            total_unadded: 0,
                            total_deleted: 0,
                            total_modified: 0,
                            total_conflicted: 0,
                            total_added: 0,
                        };
                        org_summaries.push(error_summary);
                    }
                }
            }
            
            print_org_summary(&org_summaries);
            Ok(())
        } else {
            self.run_single_org(common_args, &common::organisation(self.organisation.as_deref())?)
                .map(|_| ())
        }
    }
    
    fn run_single_org(&self, common_args: &CommonArgs, organisation: &str) -> Result<common::OrgSummary> {
        let root = common::root()?;

        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        let statuses: Result<Vec<_>> = sub_dirs.iter().map(status).collect();
        let statuses = statuses?;
        
        let statuses: Vec<_> = statuses
            .into_iter()
            .filter(|status| {
                !(self.quiet
                    && status.status.is_empty()
                    && status.status.is_ahead == 0
                    && status.status.is_behind == 0)
            })
            .collect();

        if let Some(OutputFormat::Json) = common_args.format {
            println!("{}", json!(statuses));
        } else {
            let rows = to_rows(&statuses, self.verbose);
            let table = to_table(&rows);
            table.printstd();
        }

        // Lag organizasjon-sammandrag med same statistikk som summarize
        let mut unpushed_repo_count = 0;
        let mut uncommited_repo_count = 0;
        let mut total_unadded = 0;
        let mut total_deleted = 0;
        let mut total_modified = 0;
        let mut total_conflicted = 0;
        let mut total_added = 0;

        for status in &statuses {
            if !status.status.is_empty() {
                uncommited_repo_count += 1;
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
        
        Ok(common::OrgSummary {
            name: organisation.to_string(),
            total_repos: statuses.len(),
            unpushed_repo_count,
            uncommited_repo_count,
            total_unadded,
            total_deleted,
            total_modified,
            total_conflicted,
            total_added,
        })
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

fn to_table(statuses: &[StatusRow]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(
        row!["Repo", "branch", r -> "±origin", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"],
    );
    table
}

fn to_rows(statuses: &[RepoStatus], verbose: bool) -> Vec<StatusRow> {
    let mut rows: Vec<_> = statuses.iter().flat_map(|s| s.to_rows(verbose)).collect();
    rows.append(&mut to_total_summarize(statuses));
    rows
}

fn to_total_summarize(statuses: &[RepoStatus]) -> Vec<StatusRow> {
    let mut rows = vec![StatusRow::TitleSeperation, StatusRow::SummarizeTitle];
    let total = statuses.len().to_string();
    let mut unpushed_repo_count: usize = 0;
    let mut uncommited_repo_count: usize = 0;
    let mut total_unadded: usize = 0;
    let mut total_deleted: usize = 0;
    let mut total_modified: usize = 0;
    let mut total_conflicted: usize = 0;
    let mut total_added: usize = 0;

    for status in statuses {
        if !status.status.is_empty() {
            uncommited_repo_count += 1;
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
        uncommited_repo_count: uncommited_repo_count.to_string(),
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
        uncommited_repo_count: String,
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
        uncommited_repo_count: String,
        total_unadded: String,
        total_deleted: String,
        total_modified: String,
        total_conflicted: String,
        total_added: String,
    },
    RepoSeperation,
    TitleSeperation,
    SummarizeTitle,
    OrgSummarizeTitle,
}

impl StatusRow {
    fn to_row(&self) -> Row {
        match self {
            StatusRow::RepoSeperation => row!["--------------"],
            StatusRow::TitleSeperation => row!["================"],
            StatusRow::FileDetail { status, path } => row![r => status, path],
            StatusRow::SummarizeAll {
                total,
                unpushed_repo_count,
                uncommited_repo_count,
                total_unadded,
                total_deleted,
                total_modified,
                total_conflicted,
                total_added,
            } => {
                row![total, uncommited_repo_count, r -> unpushed_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added]
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
            StatusRow::OrgSummarizeTitle => {
                row!["Organisation", "#repos", "±origin", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"]
            }
            StatusRow::OrgSummarize {
                org_name,
                total_repos,
                unpushed_repo_count,
                uncommited_repo_count: _,
                total_unadded,
                total_deleted,
                total_modified,
                total_conflicted,
                total_added,
            } => {
                row![org_name, total_repos, r -> unpushed_repo_count, r -> total_unadded, r -> total_deleted, r -> total_modified, r -> total_conflicted, r -> total_added]
            }
        }
    }
}

pub fn print_org_summary(summaries: &[common::OrgSummary]) {
    let mut rows = vec![StatusRow::TitleSeperation, StatusRow::OrgSummarizeTitle];
    
    for summary in summaries {
        let org_row = StatusRow::OrgSummarize {
            org_name: summary.name.clone(),
            total_repos: summary.total_repos.to_string(),
            unpushed_repo_count: summary.unpushed_repo_count.to_string(),
            uncommited_repo_count: summary.uncommited_repo_count.to_string(),
            total_unadded: summary.total_unadded.to_string(),
            total_deleted: summary.total_deleted.to_string(),
            total_modified: summary.total_modified.to_string(),
            total_conflicted: summary.total_conflicted.to_string(),
            total_added: summary.total_added.to_string(),
        };
        rows.push(org_row);
    }
    
    let table = to_org_summary_table(&rows);
    println!("\n=== All org summary ===");
    table.printstd();
}

fn to_org_summary_table(statuses: &[StatusRow]) -> Table {
    let rows: Vec<_> = statuses.par_iter().map(|s| s.to_row()).collect();
    let mut table = Table::init(rows);
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(
        row!["Organisation", "#repos", r -> "±origin", r -> "U", r -> "D", r -> "M", r -> "C", r -> "A"],
    );
    table
}
