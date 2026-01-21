use super::patch_file::*;
use crate::commands::common;
use crate::commands::models::ExistDirectory;
use crate::commands::models::template::*;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::*;
use git2::Repository;
use prettytable::{Cell, Row, Table, cell, format, row};
use std::fs::{File, create_dir_all, write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

#[derive(Debug)]
enum ApplyStatus {
    Applied,
    Committed,
    Aborted,
    NotReady(String),
    Failed(anyhow::Error),
}

struct Status {
    repo: String,
    status: ApplyStatus,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![
            cell!(b -> &self.repo),
            self.status_cell(),
            self.error_cell(),
        ])
    }

    fn status_cell(&self) -> Cell {
        match &self.status {
            ApplyStatus::Applied => cell!(Fg -> "Applied"),
            ApplyStatus::Committed => cell!(Fg -> "Committed"),
            ApplyStatus::Aborted => cell!(Fy -> "Aborted"),
            ApplyStatus::NotReady(_) => cell!(Fy -> "Not Ready"),
            ApplyStatus::Failed(_) => cell!(Fr -> "Failed"),
        }
    }

    fn error_cell(&self) -> Cell {
        match &self.status {
            ApplyStatus::Failed(e) => {
                let msg = format!("{}", e);
                let lines = common::sub_strings(&msg, 80);
                cell!(Fr -> lines.join("\n"))
            }
            ApplyStatus::NotReady(reason) => {
                let lines = common::sub_strings(reason, 80);
                cell!(Fy -> lines.join("\n"))
            }
            _ => cell!(""),
        }
    }

    fn is_success(&self) -> bool {
        matches!(
            self.status,
            ApplyStatus::Applied | ApplyStatus::Committed | ApplyStatus::Aborted
        )
    }

    fn is_not_ready(&self) -> bool {
        matches!(self.status, ApplyStatus::NotReady(_))
    }

    fn has_error(&self) -> bool {
        matches!(self.status, ApplyStatus::Failed(_))
    }
}

/// Apply changes from template to all prject that match the regex
#[derive(Debug, Parser)]
pub struct ApplyArgs {
    /// Directory of template project
    #[arg(long, short)]
    pub template: ExistDirectory,
    /// Target owner (organisation or user) name
    #[arg(long, short)]
    pub owner: Option<String>,
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
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        if self.finish && self.abort {
            println!("You cannot provide both \"--continue\" and \"--abort\" at the same time");
            return Ok(());
        }

        let root = common::root()?;
        let owner = common::owner(self.owner.as_deref())?;
        let target_dirs = common::read_dirs_for_owner(owner.as_str(), &root, self.regex.as_ref())?;

        if target_dirs.is_empty() {
            println!(
                "There are no local repositories in {} that match the pattern {:?}",
                owner, self.regex
            );
            return Ok(());
        }

        if self.finish {
            let skip_ci = !self.force_ci;
            let statuses = common::process_with_progress(
                "Committing",
                &target_dirs,
                |dir| {
                    let repo_name = path::dir_name(dir).unwrap_or_default();
                    let status = match continue_apply(dir, skip_ci) {
                        Ok(_) => ApplyStatus::Committed,
                        Err(e) => {
                            let msg = format!("{}", e);
                            if msg.contains("not clean") || msg.contains("not ready") {
                                ApplyStatus::NotReady(msg)
                            } else {
                                ApplyStatus::Failed(e)
                            }
                        }
                    };
                    Status {
                        repo: repo_name,
                        status,
                    }
                },
                |s| s.repo.clone(),
            );
            summarize_continue(&statuses);
        } else if self.abort {
            let statuses = common::process_with_progress(
                "Aborting",
                &target_dirs,
                |dir| {
                    let repo_name = path::dir_name(dir).unwrap_or_default();
                    let status = match abort_apply(dir) {
                        Ok(_) => ApplyStatus::Aborted,
                        Err(e) => ApplyStatus::Failed(e),
                    };
                    Status {
                        repo: repo_name,
                        status,
                    }
                },
                |s| s.repo.clone(),
            );
            summarize_abort(&statuses);
        } else {
            let template_delta =
                TemplateDelta::get(&self.template.path.join(".gut/template.toml"))?;
            let template_path = &self.template.path;
            let optional = self.optional;

            let statuses = common::process_with_progress(
                "Applying",
                &target_dirs,
                |dir| {
                    let repo_name = path::dir_name(dir).unwrap_or_default();
                    let status = match start_apply(template_path, &template_delta, dir, optional) {
                        Ok(_) => ApplyStatus::Applied,
                        Err(e) => ApplyStatus::Failed(e),
                    };
                    Status {
                        repo: repo_name,
                        status,
                    }
                },
                |s| s.repo.clone(),
            );
            summarize_apply(&statuses);
        }

        Ok(())
    }
}

/// do clean -f and reset --hard
/// Remove temp directory
fn abort_apply(target_dir: &PathBuf) -> Result<()> {
    // git clean -f && git reset --hard
    clean_git_dir(target_dir)?;
    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    if template_apply_dir.exists() {
        path::remove_path(template_apply_dir)?;
    }
    Ok(())
}

/// - Check if there is no *.rej, *.orig
/// - Check if everthing is added
/// - rewrite target delta file
/// - will remove template_apply directory
fn continue_apply(target_dir: &PathBuf, skip_ci: bool) -> Result<()> {
    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let apply_status_path = &template_apply_dir.join("APPLYING");

    // check if status exists
    if !apply_status_path.exists() {
        return Err(anyhow!("There are no ongoing template apply changes."));
    }

    // check if repo is clean
    // and everything is added
    let target_repo = git::open::open(target_dir)?;
    let status = git::status(&target_repo, false)?;
    if !status.is_not_dirty() {
        return Err(anyhow!(
            "Target repo is not clean. Please use \"git add\" to add all changes before continue."
        ));
    }

    // rewrite delta file
    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let new_delta = TargetDelta::get(&template_apply_dir.join("temp_target_delta.toml"))?;
    let delta_path = &target_dir.join(".gut/delta.toml");
    new_delta.save(delta_path)?;
    let mut index = target_repo.index()?;
    index.add_path(Path::new(".gut/delta.toml"))?;
    let message = if skip_ci {
        format!("Apply changes {:?}\n\n[skip ci]", new_delta.rev_id)
    } else {
        format!("Apply changes {:?}", new_delta.rev_id)
    };

    // commit everything
    git::commit_index(&target_repo, &mut index, message.as_str())?;

    // remove temp dir
    path::remove_path(template_apply_dir)?;

    Ok(())
}

/// - check if there is APPLYING file in template_appy directory
/// - Check target delta file
/// - Read target delta file => {rev_id, template_sha}
/// - Create directory .git/gut/template_appy/
/// - Create a file inside that directory: APPLYING
/// - Traversal template repo to get current_sha, and last_sha
/// - get diff
/// - get patch_file
/// - transform patch file
/// - write patch file to .git/gut/template_appy/patch.diff
/// - apply patch command in target repo
/// - Done.
fn start_apply(
    template_dir: &PathBuf,
    template_delta: &TemplateDelta,
    target_dir: &PathBuf,
    optional: bool,
) -> Result<()> {
    //println!("Start Applying for {:?}", target_dir);

    let target_delta = TargetDelta::get(&target_dir.join(".gut/delta.toml"))?;

    // check if repo is clean
    // If repo is not clean stop
    let target_repo = git::open::open(target_dir)?;
    let status = git::status(&target_repo, false)?;

    if !status.is_empty() {
        return Err(anyhow!(
            "Target repo is not clean. Please clean or commit new changes before applying changes from template."
        ));
    }

    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let apply_status_path = &template_apply_dir.join("APPLYING");

    // check if status exists
    if apply_status_path.exists() {
        return Err(anyhow!(
            "We are in the middle of an apply process. Please use the \"--abort\" or \"--continue\" option"
        ));
    }

    // create template_apply dir
    create_dir_all(template_apply_dir)?;
    // write status file to mark process as on going
    File::create(apply_status_path)?;

    let template_repo = git::open::open(template_dir)?;

    let temp_current_sha = git::head_sha(&template_repo)?;
    let temp_last_sha = previous_template_sha(&template_repo, &target_delta)?;

    let generate_files = template_delta.generate_files(optional);
    let diff = git::diff::diff_trees(
        &template_repo,
        temp_last_sha.as_str(),
        temp_current_sha.as_str(),
    )?;

    let patch_files = diff_to_patch(&diff)?;

    //for p in &patch_files {
    //println!("======================");
    //println!("{:?}", p);
    //}

    let patch_files: Vec<_> = patch_files
        .into_iter()
        .filter(|p| generate_files.contains(&p.new_file))
        .collect();

    let target_patch_files = patch_files
        .iter()
        .map(|p| p.apply_patterns(&target_delta.replacements));
    let target_patch_files: Result<Vec<_>> = target_patch_files.into_iter().collect();

    let diff_path = &template_apply_dir.join("patch.diff");
    write(diff_path, to_content(&target_patch_files?))?;

    // Create temp_target_delta.toml BEFORE patching, so it exists even if patching fails
    let update_target_delta = target_delta.update(template_delta.rev_id, temp_current_sha.as_str());
    update_target_delta.save(&template_apply_dir.join("temp_target_delta.toml"))?;

    execute_patch(diff_path.to_str().unwrap(), target_dir)?;

    Ok(())
}

fn previous_template_sha(template_repo: &Repository, target_delta: &TargetDelta) -> Result<String> {
    let sha_from_target = &target_delta.template_sha;
    if git::get_commit(template_repo, sha_from_target).is_ok() {
        return Ok(sha_from_target.to_string());
    }

    //revwalk
    let mut revwalk = template_repo.revwalk()?;
    revwalk.push_head()?;
    let mut sort = git2::Sort::TIME;
    sort.insert(git2::Sort::REVERSE);
    revwalk.set_sorting(sort)?;
    for rev in revwalk {
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

fn execute_patch(patch_file: &str, dir: &PathBuf) -> Result<Output> {
    let output = Command::new("patch")
        .arg("-p1")
        .arg("-f")
        .arg("-i")
        .arg(patch_file)
        .current_dir(dir)
        .output()
        .expect("failed to execute process");

    log::debug!("Patch result {:?} at {:?}: {:?}", patch_file, dir, output);
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = str::from_utf8(&output.stderr)
            .unwrap_or("unknown error")
            .trim();
        let stdout = str::from_utf8(&output.stdout).unwrap_or("").trim();

        // patch often writes errors to stdout, so include both
        let error_msg = if !stderr.is_empty() && !stdout.is_empty() {
            format!("{}\n{}", stdout, stderr)
        } else if !stderr.is_empty() {
            stderr.to_string()
        } else if !stdout.is_empty() {
            stdout.to_string()
        } else {
            "patching failed with no output".to_string()
        };

        Err(anyhow!("patch: {}", error_msg))
    }
}

fn clean_git_dir(dir: &PathBuf) -> Result<()> {
    Command::new("git")
        .arg("clean")
        .arg("-f")
        .current_dir(dir)
        .output()
        .expect("failed to execute process");

    Command::new("git")
        .arg("reset")
        .arg("--hard")
        .current_dir(dir)
        .output()
        .expect("failed to execute process");

    Ok(())
}

fn to_table(statuses: &[Status]) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status", "Error"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn summarize_apply(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let successes: Vec<_> = statuses.iter().filter(|s| s.is_success()).collect();
    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nApplied template to {} repos!", successes.len());
        println!("{}", msg.green());
        println!(
            "{}",
            "Resolve any conflicts, then use \"git add\" and run with --continue.".yellow()
        );
    }

    if errors.is_empty() {
        println!("\nThere are no errors!");
    } else {
        let msg = format!(
            "\nThere were {} errors. Use --abort to reset failed repos.",
            errors.len()
        );
        println!("{}", msg.red());
    }
}

fn summarize_abort(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let successes: Vec<_> = statuses.iter().filter(|s| s.is_success()).collect();
    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nAborted {} repos!", successes.len());
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere are no errors!");
    } else {
        let msg = format!("\nThere were {} errors.", errors.len());
        println!("{}", msg.red());
    }
}

fn summarize_continue(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let successes: Vec<_> = statuses.iter().filter(|s| s.is_success()).collect();
    let not_ready: Vec<_> = statuses.iter().filter(|s| s.is_not_ready()).collect();
    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nCommitted {} repos!", successes.len());
        println!("{}", msg.green());
    }

    if !not_ready.is_empty() {
        let msg = format!(
            "\n{} repos are not ready (resolve conflicts and use \"git add\" first).",
            not_ready.len()
        );
        println!("{}", msg.yellow());
    }

    if errors.is_empty() && not_ready.is_empty() {
        println!("\nThere are no errors!");
    } else if !errors.is_empty() {
        let msg = format!("\nThere were {} errors.", errors.len());
        println!("{}", msg.red());
    }
}
