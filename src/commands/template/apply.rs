use super::patch_file::*;
use crate::commands::common;
use crate::commands::models::template::*;
use crate::commands::models::ExistDirectory;
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{anyhow, Result};
use clap::Parser;
use git2::Repository;
use std::fs::{create_dir_all, write, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;

/// Apply changes from template to all prject that match the regex
#[derive(Debug, Parser)]
pub struct ApplyArgs {
    /// Directory of template project
    #[arg(long, short)]
    pub template: ExistDirectory,
    /// Target organisation name
    #[arg(long, short, default_value = "divvun")]
    pub organisation: String,
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
    /// Skip CI
    #[arg(long)]
    pub skip_ci: bool,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        if self.finish && self.abort {
            println!("You cannot provide both \"--continue\" and \"--abort\" at the same time");
            return Ok(());
        }

        let root = common::root()?;
        let target_dirs =
            common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        if self.finish {
            // finish apply process
            for dir in target_dirs {
                match continue_apply(&dir, self.skip_ci) {
                    Ok(_) => println!("Apply changes finish successfully"),
                    Err(e) => println!("Apply changes finish failed because {:?}", e),
                }
            }
        } else if self.abort {
            // finish apply process
            for dir in target_dirs {
                match abort_apply(&dir) {
                    Ok(_) => println!("Abort Apply process success"),
                    Err(e) => println!("Abort Apply failed because {:?}", e),
                }
            }
        } else {
            // start apply process
            let template_delta =
                TemplateDelta::get(&self.template.path.join(".gut/template.toml"))?;

            println!("template delta {:?}", template_delta);

            for dir in target_dirs {
                match start_apply(&self.template.path, &template_delta, &dir, self.optional) {
                    Ok(_) => println!("Applied changes success. Please resolve conflict and use \"git add\" to add all changes before continue."),
                    Err(e) => println!("Applied changes failed {:?}\n Please use \"--abort\" option to abort the process.", e),
                }
            }
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

    // check if status is exist
    if !apply_status_path.exists() {
        return Err(anyhow!("There is no on going template apply changes."));
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
    println!("Start Applying for {:?}", target_dir);

    let target_delta = TargetDelta::get(&target_dir.join(".gut/delta.toml"))?;

    // check if repo is clean
    // If repo is not clean stop
    let target_repo = git::open::open(target_dir)?;
    let status = git::status(&target_repo, false)?;

    if !status.is_empty() {
        return Err(anyhow!("Target repo is not clean. Please clean or commit new changes before applying changes from template."));
    }

    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let apply_status_path = &template_apply_dir.join("APPLYING");

    // check if status is exist
    if apply_status_path.exists() {
        return Err(anyhow!("We are in middle of an applying process. Please use \"--abort\" or \"--continue\" option"));
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
    execute_patch(diff_path.to_str().unwrap(), target_dir)?;

    let update_target_delta = target_delta.update(template_delta.rev_id, temp_current_sha.as_str());
    update_target_delta.save(&template_apply_dir.join("temp_target_delta.toml"))?;

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
            if let Ok(delta) = toml::from_str::<TemplateDelta>(content) {
                if delta.rev_id == target_delta.rev_id {
                    return Ok(rev.to_string());
                }
            }
        }
    }
    Err(anyhow!("Cannot find the commit of previous rev_id"))
}

fn execute_patch(patch_file: &str, dir: &PathBuf) -> Result<Output> {
    let output = Command::new("patch")
        .arg("-p1")
        .arg("-i")
        .arg(patch_file)
        .current_dir(dir)
        .output()
        .expect("failed to execute process");

    log::debug!("Patch result {:?} at {:?}: {:?}", patch_file, dir, output);

    Ok(output)
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
