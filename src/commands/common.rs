use crate::commands::topic_helper;
use crate::config::Config;
use crate::path;
use anyhow::{Context, Result, anyhow};
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Table, format, row};
use rayon::prelude::*;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

use crate::github;
use crate::github::{NoReposFound, RemoteRepo, Unauthorized};

use crate::filter::{Filter, Filterable};
use crate::user::User;

/// Trait for types that can create error placeholders for failed organizations
pub trait ErrorPlaceholder {
    fn error_placeholder(org_name: &str) -> Self;
}

#[derive(Debug, Clone)]
pub struct OrgResult {
    pub org_name: String,
    pub total_repos: usize,
    pub successful_repos: usize,
    pub failed_repos: usize,
    pub dirty_repos: usize,
}

#[derive(Debug, Clone)]
pub struct OrgSummary {
    pub name: String,
    pub total_repos: usize,
    pub unpushed_repo_count: usize,
    pub uncommitted_repo_count: usize,
    pub total_unadded: usize,
    pub total_deleted: usize,
    pub total_modified: usize,
    pub total_conflicted: usize,
    pub total_added: usize,
}

impl ErrorPlaceholder for OrgSummary {
    fn error_placeholder(org_name: &str) -> Self {
        Self {
            name: org_name.to_string(),
            total_repos: 0,
            unpushed_repo_count: 0,
            uncommitted_repo_count: 0,
            total_unadded: 0,
            total_deleted: 0,
            total_modified: 0,
            total_conflicted: 0,
            total_added: 0,
        }
    }
}

impl ErrorPlaceholder for OrgResult {
    fn error_placeholder(org_name: &str) -> Self {
        Self::new(org_name)
    }
}

impl OrgResult {
    pub fn new(org_name: &str) -> Self {
        Self {
            org_name: org_name.to_string(),
            total_repos: 0,
            successful_repos: 0,
            failed_repos: 0,
            dirty_repos: 0,
        }
    }

    pub fn add_success(&mut self) {
        self.total_repos += 1;
        self.successful_repos += 1;
    }

    pub fn add_failure(&mut self) {
        self.total_repos += 1;
        self.failed_repos += 1;
    }
}

/// Print a summary table for OrgResult slices with a customizable success column label
pub fn print_org_result_summary(summaries: &[OrgResult], success_column_label: &str) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row![
        "Owner",
        "#repos",
        success_column_label,
        "Failed"
    ]);

    let mut total_repos = 0;
    let mut total_success = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_success += summary.successful_repos;
        total_failed += summary.failed_repos;
    }

    table.add_empty_row();
    table.add_row(row!["TOTAL", r -> total_repos, r -> total_success, r -> total_failed]);

    println!("\n=== All owner summary ===");
    table.printstd();
}

/// Run a command against all owners or a single one, printing OrgResult summary with custom label
pub fn run_for_orgs<F>(
    all_orgs: bool,
    organisation_opt: Option<&str>,
    run_fn: F,
    success_column_label: &str,
) -> Result<()>
where
    F: Fn(&str) -> Result<OrgResult>,
{
    if all_orgs {
        let organizations = get_all_organizations()?;
        if organizations.is_empty() {
            println!("No owners found in root directory");
            return Ok(());
        }

        let mut summaries = Vec::new();

        for org in &organizations {
            println!("\n=== Processing owner: {} ===", org);

            match run_fn(org) {
                Ok(summary) => {
                    summaries.push(summary);
                }
                Err(e) => {
                    println!("Failed to process owner '{}': {:?}", org, e);
                    summaries.push(OrgResult::error_placeholder(org));
                }
            }
        }

        print_org_result_summary(&summaries, success_column_label);

        Ok(())
    } else {
        let org = organisation(organisation_opt)?;
        run_fn(&org)?;
        Ok(())
    }
}

/// Run a command against all owners or a single one with custom summary printer
pub fn run_for_orgs_with_summary<F, R>(
    all_orgs: bool,
    organisation_opt: Option<&str>,
    run_fn: F,
    print_summary_fn: fn(&[R]),
) -> Result<()>
where
    F: Fn(&str) -> Result<R>,
    R: ErrorPlaceholder,
{
    if all_orgs {
        let organizations = get_all_organizations()?;
        if organizations.is_empty() {
            println!("No owners found in root directory");
            return Ok(());
        }

        let mut summaries = Vec::new();

        for org in &organizations {
            println!("\n=== Processing owner: {} ===", org);

            match run_fn(org) {
                Ok(summary) => {
                    summaries.push(summary);
                }
                Err(e) => {
                    println!("Failed to process owner '{}': {:?}", org, e);
                    summaries.push(R::error_placeholder(org));
                }
            }
        }

        print_summary_fn(&summaries);

        Ok(())
    } else {
        let org = organisation(organisation_opt)?;
        run_fn(&org)?;
        Ok(())
    }
}

pub fn query_and_filter_repositories(
    org: &str,
    regex: Option<&Filter>,
    token: &str,
) -> Result<Vec<RemoteRepo>> {
    let remote_repos = remote_repos(token, org)?;
    let mut result = RemoteRepo::filter_with_option(remote_repos, regex);
    result.sort();
    Ok(result)
}

pub fn user() -> Result<User> {
    User::from_config()
        .context("Cannot get user token from the config file. Run `gut init` with a valid token")
}

pub fn root() -> Result<String> {
    Config::root()
        .context("Cannot read the config file. Run `gut init` with valid token and root directory")
}

pub fn user_token() -> Result<String> {
    User::token()
        .context("Cannot get user token from the config file. Run `gut init` with a valid token")
}

pub fn organisation(opt: Option<&str>) -> Result<String> {
    match opt {
        Some(s) => Ok(s.to_string()),
        None => {
            let config = Config::from_file()?;
            match config.default_owner {
                Some(o) => Ok(o),
                None => anyhow::bail!(
                    "You need to provide an owner (organization or user) or set a default owner with init/set default owner command."
                ),
            }
        }
    }
}

pub fn use_https() -> Result<bool> {
    let config = Config::from_file()?;
    Ok(config.use_https)
}

fn remote_repos(token: &str, org: &str) -> Result<Vec<RemoteRepo>> {
    match github::list_owner_repos(token, org).context("When fetching repositories") {
        Ok(repos) => Ok(repos),
        Err(e) => {
            if e.downcast_ref::<NoReposFound>().is_some() {
                anyhow::bail!("No repositories found");
            }
            if e.downcast_ref::<Unauthorized>().is_some() {
                anyhow::bail!("User token invalid. Run `gut init` with a valid token");
            }
            Err(e)
        }
    }
}

pub fn read_dirs_for_org(org: &str, root: &str, filter: Option<&Filter>) -> Result<Vec<PathBuf>> {
    let target_dir = path::local_path_org(org, root)?;

    let result = match filter {
        Some(f) => read_dirs_with_filter(&target_dir, f),
        None => read_dirs(&target_dir),
    };

    match result {
        Ok(mut vec) => {
            vec.sort();
            Ok(vec)
        }
        Err(e) => Err(anyhow!(
            "Cannot read sub directories for owner {} \"{}\" because {:?}",
            target_dir.display(),
            org,
            e
        )),
    }
}

/// Get repository directories for an owner (organization or user).
///
/// If `topic` is Some, queries GitHub API to filter by topic (and optionally regex).
/// If `topic` is None, uses local directories only (faster, no API call).
pub fn get_repo_dirs(
    org: &str,
    topic: Option<&String>,
    regex: Option<&Filter>,
    token: &str,
    root: &str,
) -> Result<Vec<PathBuf>> {
    if topic.is_some() {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message("Querying GitHub for matching repositories...");
        spinner.enable_steady_tick(Duration::from_millis(100));

        let all_repos = topic_helper::query_repositories_with_topics(org, token)?;

        spinner.finish_and_clear();

        let filtered = topic_helper::filter_repos(&all_repos, topic, regex);
        let root_path = Path::new(root);
        Ok(filtered
            .into_iter()
            .map(|r| root_path.join(org).join(&r.repo.name))
            .collect())
    } else {
        read_dirs_for_org(org, root, regex)
    }
}

/// Filter directory's name by regex
pub fn read_dirs_with_filter(path: &Path, filter: &Filter) -> Result<Vec<PathBuf>> {
    let dirs = read_dirs(path)?;
    Ok(PathBuf::filter(dirs, filter))
}

/// Read all dirs inside a path
/// Filter directories
fn read_dirs(path: &Path) -> Result<Vec<PathBuf>> {
    let entries = path.read_dir()?;
    let dirs = entries
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|x| x.is_dir())
        .collect();
    Ok(dirs)
}

/// Checks if a directory contains at least one git repository
fn contains_git_repos(path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };

    entries.filter_map(|e| e.ok()).any(|entry| {
        entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            && entry.path().join(".git").is_dir()
    })
}

pub fn get_all_organizations() -> Result<Vec<String>> {
    let root = root()?;
    let root_path = Path::new(&root);

    if !root_path.exists() {
        return Ok(Vec::new());
    }

    let mut organizations: Vec<String> = std::fs::read_dir(root_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter_map(|entry| {
            let name = entry.file_name();
            let name_str = name.to_str()?;

            // Skip hidden directories and validate it contains git repos
            if !name_str.starts_with('.') && contains_git_repos(&entry.path()) {
                Some(name_str.to_string())
            } else {
                None
            }
        })
        .collect();

    organizations.sort();
    Ok(organizations)
}

pub fn confirm(prompt: &str, key: &str) -> Result<bool> {
    let confirm = Input::<String>::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact()?;
    Ok(confirm == key)
}

pub fn ask_for(prompt: &str) -> Result<String> {
    let confirm = Input::<String>::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact()?;
    Ok(confirm)
}

pub fn apply_script(dir: &PathBuf, script: &str) -> Result<Output> {
    let output = execute_script(script, dir)?;
    if output.status.success() {
        Ok(output)
    } else {
        let err_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| format!("Cannot execute the script {}", script));
        Err(anyhow!(err_message))
    }
}

fn execute_script(script: &str, dir: &PathBuf) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", script])
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    };

    //log::debug!("Script result {:?}", output);

    Ok(output)
}

pub fn sub_strings(string: &str, sub_len: usize) -> Vec<&str> {
    let mut subs = Vec::with_capacity(string.len() / sub_len);
    let mut iter = string.chars();
    let mut pos = 0;

    while pos < string.len() {
        let mut len = 0;
        for ch in iter.by_ref().take(sub_len) {
            len += ch.len_utf8();
        }
        subs.push(&string[pos..pos + len]);
        pos += len;
    }
    subs
}

/// Process items in parallel with a progress bar
///
/// - `title`: text shown before the progress bar (e.g., "Cloning", "Fetching")
/// - `items`: the collection to iterate over
/// - `process`: function that processes each item and returns a result
/// - `name_fn`: function that extracts a display name from the result (shown in progress bar)
pub fn process_with_progress<T, R, F, N>(title: &str, items: &[T], process: F, name_fn: N) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
    N: Fn(&R) -> String + Sync + Send,
{
    let pb = ProgressBar::new(items.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix} {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_prefix(title.to_string());

    let results: Vec<R> = items
        .par_iter()
        .map(|item| {
            let result = process(item);
            pb.set_message(name_fn(&result));
            pb.inc(1);
            result
        })
        .collect();

    pb.finish_and_clear();

    results
}
