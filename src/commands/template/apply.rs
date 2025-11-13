use crate::cli::Args as CommonArgs;
use crate::commands::common;
use crate::commands::models::ExistDirectory;
use crate::commands::models::template::*;
use crate::commands::patterns::generate_string;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Error, Result, anyhow};
use clap::Parser;
use colored::*;
use git2::{DiffOptions, Repository};
use prettytable::{Cell, Row, Table, cell, format, row};
use rayon::prelude::*;
use std::fs::{File, create_dir_all, create_dir_all as fs_create_dir_all, write};
use std::path::{Path, PathBuf};
use std::str;

/// Apply changes from template to all projects that match the regex
#[derive(Debug, Parser)]
pub struct ApplyArgs {
    /// Directory of template project
    #[arg(long, short)]
    pub template: ExistDirectory,
    /// Target organisation name
    #[arg(long, short)]
    pub organisation: Option<String>,
    /// Optional regex to filter repositories
    #[arg(long, short)]
    pub regex: Option<Filter>,
    /// Flag to finish apply changes process
    #[arg(long = "continue")]
    pub finish: bool,
    /// Flag to abort apply changes process
    #[arg(long)]
    pub abort: bool,
    /// Flag to include optional files
    #[arg(long)]
    pub optional: bool,
    /// Force CI build -- automatically skips otherwise
    #[arg(long)]
    pub force_ci: bool,
    /// Override template baseline SHA (instead of using delta.toml)
    #[arg(long)]
    pub from_sha: Option<String>,
}

impl ApplyArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        if self.finish && self.abort {
            println!("You cannot provide both \"--continue\" and \"--abort\" at the same time");
            return Ok(());
        }

        let root = common::root()?;
        let organisation = common::organisation(self.organisation.as_deref())?;
        let target_dirs =
            common::read_dirs_for_org(organisation.as_str(), &root, self.regex.as_ref())?;

        if target_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} that matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        if self.finish {
            let statuses: Vec<_> = target_dirs
                .par_iter()
                .map(|dir| process_continue(dir, !self.force_ci))
                .collect();
            summarize_continue(&statuses);
        } else if self.abort {
            let statuses: Vec<_> = target_dirs.par_iter().map(process_abort).collect();
            summarize_abort(&statuses);
        } else {
            let template_delta =
                TemplateDelta::get(&self.template.path.join(".gut/template.toml"))?;

            let statuses: Vec<_> = target_dirs
                .par_iter()
                .map(|dir| {
                    process_start(
                        &self.template.path,
                        &template_delta,
                        dir,
                        self.optional,
                        self.from_sha.as_deref(),
                    )
                })
                .collect();
            summarize_start(&statuses);
        }

        Ok(())
    }
}

// Status structs for each phase

struct StartStatus {
    repo: String,
    result: Result<(), Error>,
}

impl StartStatus {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo), self.status_cell()])
    }

    fn status_cell(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Ready for continue"),
            Err(_) => cell!(Frr -> "Failed"),
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

struct ContinueStatus {
    repo: String,
    result: Result<(), Error>,
}

impl ContinueStatus {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo), self.status_cell()])
    }

    fn status_cell(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Success"),
            Err(_) => cell!(Frr -> "Failed"),
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

struct AbortStatus {
    repo: String,
    result: Result<(), Error>,
}

impl AbortStatus {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo), self.status_cell()])
    }

    fn status_cell(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Aborted"),
            Err(_) => cell!(Frr -> "Failed"),
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

// Processing functions

fn process_start(
    template_dir: &PathBuf,
    template_delta: &TemplateDelta,
    target_dir: &PathBuf,
    optional: bool,
    from_sha: Option<&str>,
) -> StartStatus {
    let repo_name = path::dir_name(target_dir)
        .unwrap_or_else(|_| target_dir.to_str().unwrap_or("unknown").to_string());

    log::info!("Applying template to {}", repo_name);
    let result = start_apply(template_dir, template_delta, target_dir, optional, from_sha);

    StartStatus {
        repo: repo_name,
        result,
    }
}

fn process_continue(target_dir: &PathBuf, skip_ci: bool) -> ContinueStatus {
    let repo_name = path::dir_name(target_dir)
        .unwrap_or_else(|_| target_dir.to_str().unwrap_or("unknown").to_string());

    log::info!("Continuing apply for {}", repo_name);
    let result = continue_apply(target_dir, skip_ci);

    ContinueStatus {
        repo: repo_name,
        result,
    }
}

fn process_abort(target_dir: &PathBuf) -> AbortStatus {
    let repo_name = path::dir_name(target_dir)
        .unwrap_or_else(|_| target_dir.to_str().unwrap_or("unknown").to_string());

    log::info!("Aborting apply for {}", repo_name);
    let result = abort_apply(target_dir);

    AbortStatus {
        repo: repo_name,
        result,
    }
}

// Summary functions

fn summarize_start(statuses: &[StartStatus]) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!(
            "\nTemplate applied to {} repos. Run with --continue to finalize (or --abort to cancel)",
            successes.len()
        );
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!("There are {} errors:", errors.len());
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

fn summarize_continue(statuses: &[ContinueStatus]) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!(
            "\nSuccessfully applied changes to {} repos!",
            successes.len()
        );
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!("There are {} errors:", errors.len());
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

fn summarize_abort(statuses: &[AbortStatus]) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nAborted template apply for {} repos", successes.len());
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!("There are {} errors:", errors.len());
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

/// Checkout original branch, clean working directory, reset to HEAD, and remove temp directory
fn abort_apply(target_dir: &PathBuf) -> Result<()> {
    let repo = git::open::open(target_dir)?;

    // Checkout original branch
    if let Ok(original_branch) = get_original_branch_name(target_dir) {
        let _ = git::branch::checkout_local_branch(&repo, &original_branch);
    }

    // Clean untracked files and reset to HEAD using libgit2
    git::clean_working_dir(&repo)?;
    git::reset_hard_head(&repo)?;

    // Delete temporary branch if it exists
    if let Ok(branch_name) = get_temp_branch_name(target_dir) {
        if let Ok(mut branch) = repo.find_branch(&branch_name, git2::BranchType::Local) {
            branch.delete()?;
        }
    }

    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    if template_apply_dir.exists() {
        path::remove_path(template_apply_dir)?;
    }
    Ok(())
}

/// - Checkout original branch
/// - Merge temp branch into original branch
/// - Update target delta file
/// - Create final commit with [skip ci]
/// - Delete temporary branch
/// - Remove template_apply directory
fn continue_apply(target_dir: &PathBuf, skip_ci: bool) -> Result<()> {
    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let apply_status_path = &template_apply_dir.join("APPLYING");

    // Check if apply is in progress
    if !apply_status_path.exists() {
        return Err(anyhow!("There is no on going template apply changes."));
    }

    let target_repo = git::open::open(target_dir)?;

    // Get branch names
    let temp_branch_name = get_temp_branch_name(target_dir)?;
    let original_branch_name = get_original_branch_name(target_dir)?;

    // Checkout original branch
    git::branch::checkout_local_branch(&target_repo, &original_branch_name)?;

    // Merge temp branch into original branch
    let merge_status = git::merge_local(&target_repo, &temp_branch_name, false)?;

    // Handle merge result
    match merge_status {
        git::MergeStatus::MergeWithConflict => {
            return Err(anyhow!(
                "Merge conflicts detected. Please resolve conflicts, stage with 'git add', and run --continue again."
            ));
        }
        git::MergeStatus::FastForward | git::MergeStatus::NormalMerge => {
            // Merge created a commit, but we need to add delta file and amend with [skip ci]
        }
        git::MergeStatus::Nothing => {
            // No merge needed, just update delta
        }
        _ => {}
    }

    // Update delta file
    let new_delta = TargetDelta::get(&template_apply_dir.join("temp_target_delta.toml"))?;
    let delta_path = &target_dir.join(".gut/delta.toml");
    new_delta.save(delta_path)?;
    let mut index = target_repo.index()?;
    index.add_path(Path::new(".gut/delta.toml"))?;
    index.write()?;

    // Create final commit with [skip ci]
    let message = if skip_ci {
        format!("Apply changes {:?}\n\n[skip ci]", new_delta.rev_id)
    } else {
        format!("Apply changes {:?}", new_delta.rev_id)
    };
    git::commit_index(&target_repo, &mut index, message.as_str())?;

    // Delete temporary branch
    if let Ok(mut branch) = target_repo.find_branch(&temp_branch_name, git2::BranchType::Local) {
        branch.delete()?;
    }

    // Remove temp dir
    path::remove_path(template_apply_dir)?;

    Ok(())
}

/// Git-native merge approach:
/// - Check if repo is clean
/// - Create temporary branch gut/template-apply-rev-{rev_id}
/// - Checkout temp branch
/// - For each changed file in template diff:
///   - Read content from template repo
///   - Apply pattern replacements
///   - Write to working directory (with transformed path)
/// - Commit changes on temp branch
/// - Checkout original branch
/// - Merge temp branch (with conflict detection)
/// - Save updated delta to temp location
fn start_apply(
    template_dir: &PathBuf,
    template_delta: &TemplateDelta,
    target_dir: &PathBuf,
    optional: bool,
    from_sha: Option<&str>,
) -> Result<()> {
    let target_delta = TargetDelta::get(&target_dir.join(".gut/delta.toml"))?;

    // Check if repo is clean
    let target_repo = git::open::open(target_dir)?;
    let status = git::status(&target_repo, false)?;

    if !status.is_empty() {
        return Err(anyhow!(
            "Target repo is not clean. Please clean or commit new changes before applying changes from template."
        ));
    }

    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let apply_status_path = &template_apply_dir.join("APPLYING");

    // Check if apply is already in progress
    if apply_status_path.exists() {
        return Err(anyhow!(
            "We are in middle of an applying process. Please use \"--abort\" or \"--continue\" option"
        ));
    }

    // Create template_apply dir and status file
    create_dir_all(template_apply_dir)?;
    File::create(apply_status_path)?;

    let template_repo = git::open::open(template_dir)?;

    let temp_current_sha = git::head_sha(&template_repo)?;
    let temp_last_sha = if let Some(sha) = from_sha {
        // Validate that the provided SHA exists in template repo
        git::get_commit(&template_repo, sha).map_err(|_| {
            anyhow!(
                "Provided --from-sha '{}' not found in template repository",
                sha
            )
        })?;
        sha.to_string()
    } else {
        previous_template_sha(&template_repo, &target_delta, 1000)?
    };

    // Get current branch name
    let original_branch = git::branch::head_shorthand(&target_repo)?;

    // Create and checkout temporary branch
    let temp_branch_name = format!("gut/template-apply-rev-{}", template_delta.rev_id);
    git::branch::create_branch(&target_repo, &temp_branch_name, &original_branch)?;
    git::branch::checkout_local_branch(&target_repo, &temp_branch_name)?;

    // Save temp branch name and original branch name for continue/abort
    write(
        template_apply_dir.join("temp_branch_name"),
        &temp_branch_name,
    )?;
    write(
        template_apply_dir.join("original_branch_name"),
        &original_branch,
    )?;

    // Get list of files to apply
    let generate_files = template_delta.generate_files(optional);

    // Get diff between template versions
    let mut diff_opts = DiffOptions::new();
    let old_tree = git::tree::tree_from_commit_sha(&template_repo, &temp_last_sha)?;
    let new_tree = git::tree::tree_from_commit_sha(&template_repo, &temp_current_sha)?;
    let diff =
        template_repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut diff_opts))?;

    // Apply changes file by file
    diff.foreach(
        &mut |delta, _progress| {
            let new_file_path = delta.new_file().path().and_then(|p| p.to_str());

            if let Some(path_str) = new_file_path {
                // Only process files in the generate_files list
                if !generate_files.contains(&path_str.to_string()) {
                    return true;
                }

                // Apply pattern replacements to file path
                let transformed_path = match generate_string(&target_delta.replacements, path_str) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Failed to transform path {}: {:?}", path_str, e);
                        return false;
                    }
                };

                // Get file content from template repo
                let oid = delta.new_file().id();
                match template_repo.find_blob(oid) {
                    Ok(blob) => {
                        let content = blob.content();

                        // Apply pattern replacements to content
                        let transformed_content = match std::str::from_utf8(content) {
                            Ok(text) => match generate_string(&target_delta.replacements, text) {
                                Ok(t) => t.into_bytes(),
                                Err(e) => {
                                    eprintln!(
                                        "Failed to transform content of {}: {:?}",
                                        path_str, e
                                    );
                                    return false;
                                }
                            },
                            Err(_) => {
                                // Binary file, use as-is
                                content.to_vec()
                            }
                        };

                        // Write file to target repo working directory
                        let target_file_path = target_dir.join(&transformed_path);
                        if let Some(parent) = target_file_path.parent() {
                            if let Err(e) = fs_create_dir_all(parent) {
                                eprintln!("Failed to create directory {:?}: {:?}", parent, e);
                                return false;
                            }
                        }

                        if let Err(e) = write(&target_file_path, transformed_content) {
                            eprintln!("Failed to write file {:?}: {:?}", target_file_path, e);
                            return false;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to find blob for {}: {:?}", path_str, e);
                        return false;
                    }
                }
            }
            true
        },
        None,
        None,
        None,
    )?;

    // Stage all changes
    let mut index = target_repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Commit on temp branch
    let message = format!("Apply template changes rev {}", template_delta.rev_id);
    git::commit_index(&target_repo, &mut index, &message)?;

    // Stay on temp branch so user can see the changes
    // continue_apply will merge into original branch

    // Save updated delta
    let update_target_delta = target_delta.update(template_delta.rev_id, temp_current_sha.as_str());
    update_target_delta.save(&template_apply_dir.join("temp_target_delta.toml"))?;

    Ok(())
}

fn previous_template_sha(
    template_repo: &Repository,
    target_delta: &TargetDelta,
    max_iterations: usize,
) -> Result<String> {
    let sha_from_target = &target_delta.template_sha;
    if git::get_commit(template_repo, sha_from_target).is_ok() {
        return Ok(sha_from_target.to_string());
    }

    // Revwalk with iteration limit
    let mut revwalk = template_repo.revwalk()?;
    revwalk.push_head()?;
    let mut sort = git2::Sort::TIME;
    sort.insert(git2::Sort::REVERSE);
    revwalk.set_sorting(sort)?;

    let mut iterations = 0;
    for rev in revwalk {
        if iterations >= max_iterations {
            return Err(anyhow!(
                "Cannot find the commit of previous rev_id after {} iterations",
                max_iterations
            ));
        }
        iterations += 1;

        let rev = rev?;
        let commit = template_repo.find_commit(rev)?;
        let tree = commit.tree()?;

        if let Ok(entry) = tree.get_path(Path::new(".gut/template.toml")) {
            let object = entry.to_object(template_repo)?;
            let blob = object.peel_to_blob()?;
            let content = str::from_utf8(blob.content())?;
            if let Ok(delta) = toml::from_str::<TemplateDelta>(content)
                && delta.rev_id == target_delta.rev_id
            {
                return Ok(rev.to_string());
            }
        }
    }
    Err(anyhow!("Cannot find the commit of previous rev_id"))
}

fn get_temp_branch_name(target_dir: &PathBuf) -> Result<String> {
    let temp_branch_file = target_dir.join(".git/gut/template_apply/temp_branch_name");
    std::fs::read_to_string(temp_branch_file)
        .map_err(|e| anyhow!("Failed to read temp branch name: {}", e))
}

fn get_original_branch_name(target_dir: &PathBuf) -> Result<String> {
    let original_branch_file = target_dir.join(".git/gut/template_apply/original_branch_name");
    std::fs::read_to_string(original_branch_file)
        .map_err(|e| anyhow!("Failed to read original branch name: {}", e))
}
