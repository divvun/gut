use crate::config::Config;
use crate::path;
use anyhow::{Context, Result, anyhow};
use dialoguer::Input;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
pub struct StatusOrgResult {
    pub org_name: String,
    pub total_repos: usize,
    pub had_error: bool,
    pub repos_with_origin_changes: usize, // ±origin
    pub unstaged_files: usize,            // U
    pub deleted_files: usize,             // D
    pub modified_files: usize,            // M
    pub conflicted_files: usize,          // C
    pub added_files: usize,               // A
}

#[derive(Debug, Clone)]
pub struct OrgSummary {
    pub name: String,
    pub total_repos: usize,
    pub unpushed_repo_count: usize,
    pub uncommited_repo_count: usize,
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
            uncommited_repo_count: 0,
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
        Self {
            org_name: org_name.to_string(),
            total_repos: 0,
            successful_repos: 0,
            failed_repos: 0,
            dirty_repos: 0,
        }
    }
}

impl OrgResult {
    pub fn new(org_name: String) -> Self {
        Self {
            org_name,
            total_repos: 0,
            successful_repos: 0,
            failed_repos: 0,
            dirty_repos: 0,
        }
    }

    pub fn new_for_pull(org_name: String) -> Self {
        OrgResult {
            org_name,
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

impl StatusOrgResult {
    pub fn new(org_name: String) -> Self {
        StatusOrgResult {
            org_name,
            total_repos: 0,
            had_error: false,
            repos_with_origin_changes: 0,
            unstaged_files: 0,
            deleted_files: 0,
            modified_files: 0,
            conflicted_files: 0,
            added_files: 0,
        }
    }

    pub fn mark_error(&mut self) {
        self.had_error = true;
    }

    pub fn add_repo_status(&mut self, git_status: &crate::git::GitStatus) {
        self.total_repos += 1;

        // Count origin changes (ahead or behind)
        if git_status.is_ahead > 0 || git_status.is_behind > 0 {
            self.repos_with_origin_changes += 1;
        }

        // Count file changes - note: these are file counts, not repo counts
        self.unstaged_files += git_status.new.len();
        self.deleted_files += git_status.deleted.len();
        self.modified_files += git_status.modified.len();
        self.conflicted_files += git_status.conflicted.len();
        self.added_files += git_status.added.len();
    }
}

#[derive(Debug)]
pub struct AllOrgsResult {
    pub org_results: Vec<OrgResult>,
}

impl AllOrgsResult {
    pub fn new() -> Self {
        AllOrgsResult {
            org_results: Vec::new(),
        }
    }

    pub fn add_org_result(&mut self, result: OrgResult) {
        self.org_results.push(result);
    }

    pub fn print_summary(&self) {
        println!("\n=== SUMMARY FOR ALL ORGANIZATIONS ===");
        for result in &self.org_results {
            if result.failed_repos > 0 {
                println!(
                    "Organization '{}': {} repos processed, {} successful, {} failed",
                    result.org_name,
                    result.total_repos,
                    result.successful_repos,
                    result.failed_repos
                );
            } else {
                println!(
                    "Organization '{}': {} repos processed successfully",
                    result.org_name, result.total_repos
                );
            }
        }

        let total_orgs = self.org_results.len();
        let total_repos: usize = self.org_results.iter().map(|r| r.total_repos).sum();
        let total_successful: usize = self.org_results.iter().map(|r| r.successful_repos).sum();
        let total_failed: usize = self.org_results.iter().map(|r| r.failed_repos).sum();

        println!(
            "\nGRAND TOTAL: {} organizations, {} repos processed, {} successful, {} failed",
            total_orgs, total_repos, total_successful, total_failed
        );
    }
}

pub fn print_status_summary(results: &[StatusOrgResult]) {
    use prettytable::{Cell, Row, Table, format};

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    // Header row
    table.add_row(Row::new(vec![
        Cell::new("Organisation"),
        Cell::new("#repos"),
        Cell::new("±origin"),
        Cell::new("U"),
        Cell::new("D"),
        Cell::new("M"),
        Cell::new("C"),
        Cell::new("A"),
    ]));

    for result in results {
        let row = if result.had_error {
            Row::new(vec![
                Cell::new(&result.org_name),
                Cell::new("-error-"),
                Cell::new("-"),
                Cell::new("-"),
                Cell::new("-"),
                Cell::new("-"),
                Cell::new("-"),
                Cell::new("-"),
            ])
        } else {
            Row::new(vec![
                Cell::new(&result.org_name),
                Cell::new(&result.total_repos.to_string()),
                Cell::new(&result.repos_with_origin_changes.to_string()),
                Cell::new(&result.unstaged_files.to_string()),
                Cell::new(&result.deleted_files.to_string()),
                Cell::new(&result.modified_files.to_string()),
                Cell::new(&result.conflicted_files.to_string()),
                Cell::new(&result.added_files.to_string()),
            ])
        };
        table.add_row(row);
    }

    println!("\n=== All org summary ===");
    table.printstd();
}

/// Generic function to run a command against all organizations or a single one
pub fn run_for_orgs_or_single<F, R>(
    all_orgs: bool,
    organisation_opt: Option<&str>,
    run_fn: F,
    print_summary_fn: Option<fn(&[R])>,
) -> Result<()>
where
    F: Fn(&str) -> Result<R>,
    R: ErrorPlaceholder,
{
    if all_orgs {
        let organizations = get_all_organizations()?;
        if organizations.is_empty() {
            println!("No organizations found in root directory");
            return Ok(());
        }

        let mut summaries = Vec::new();

        for org in &organizations {
            println!("\n=== Processing organization: {} ===", org);

            match run_fn(org) {
                Ok(summary) => {
                    summaries.push(summary);
                }
                Err(e) => {
                    println!("Failed to process organization '{}': {:?}", org, e);
                    summaries.push(R::error_placeholder(org));
                }
            }
        }

        if let Some(print_fn) = print_summary_fn {
            print_fn(&summaries);
        }

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
            match config.default_org {
                Some(o) => Ok(o),
                None => anyhow::bail!(
                    "You need to provide an organisation or set a default organisation with init/set default organisation command."
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
    match github::list_org_repos(token, org).context("When fetching repositories") {
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
            "Cannot read sub directories for organisation {} \"{}\" because {:?}",
            target_dir.display(),
            org,
            e
        )),
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

/// Find all organizations by scanning the local filesystem root directory
pub fn get_all_organizations() -> Result<Vec<String>> {
    let root = root()?;
    let root_path = Path::new(&root);

    if !root_path.exists() {
        return Ok(Vec::new());
    }

    let mut organizations = Vec::new();

    for entry in std::fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();

        // Only consider directories
        if path.is_dir() {
            if let Some(org_name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip hidden directories and common non-org directories
                if !org_name.starts_with('.')
                    && org_name != "target"
                    && org_name != "node_modules"
                    && org_name != ".git"
                {
                    organizations.push(org_name.to_string());
                }
            }
        }
    }

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
