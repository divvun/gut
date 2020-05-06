use std::process::{Command, Output};
use crate::filter::Filter;
use crate::path;
use std::path::{Path, PathBuf};
use super::model::*;
use super::patch_file::*;
use anyhow::{anyhow, Result};
use structopt::StructOpt;
use std::fs::{write, create_dir_all, File};
use crate::git;
use crate::commands::models::ExistDirectory;

#[derive(Debug, StructOpt)]
pub struct ApplyArgs {
    /// Directory of template project
    #[structopt(long, short)]
    pub template: ExistDirectory,
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long="continue")]
    pub finish: bool,
    #[structopt(long)]
    pub abort: bool,
    #[structopt(long)]
    pub optional: bool,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        println!("Template apply args {:?}", self);

        if self.finish && self.abort {
            println!(
                "You cannot provide both \"--continue\" and \"--abort\" at the same time"
            );
            return Ok(());
        }

        // TODO use real target dirs
        let target_dirs = vec![Path::new("/Users/thanhle/dadmin/dadmin-test/lang-fr").to_path_buf()];

        if self.finish {
            // finish apply process
            for dir in target_dirs {
                match continue_apply(&self.template.path, &dir) {
                    Ok(_) => println!("Finish Apply success"),
                    Err(e) => println!("Finish Apply failed {:?}", e),
                }
            }
        } else if self.abort {
            // finish apply process
            for dir in target_dirs {
                match abort_apply(&self.template.path, &dir) {
                    Ok(_) => println!("Abort Apply success"),
                    Err(e) => println!("Abort Apply failed {:?}", e),
                }
            }
        } else {
            // start apply process
            let template_delta = TemplateDelta::get(&self.template.path.join(".gut/template.toml"))?;

            println!("template delta {:?}", template_delta);

            for dir in target_dirs {
                match start_apply(&self.template.path, &template_delta, &dir, self.optional) {
                    Ok(_) => println!("Applied success"),
                    Err(e) => println!("Applied failed {:?}", e),
                }
            }
        }

        Ok(())
    }
}

/// do clean -f and reset --hard
/// Remove temp directory
fn abort_apply(template_dir: &PathBuf, target_dir: &PathBuf) -> Result<()> {
    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    path::remove_path(template_apply_dir)?;
    // git clean -f && git reset --hard
    clean_git_dir(target_dir)?;
    Ok(())
}

/// - Check if there is no *.rej, *.orig
/// - Check if everthing is added
/// - rewrite target delta file
/// - will remove template_apply directory
fn continue_apply(template_dir: &PathBuf, target_dir: &PathBuf) -> Result<()> {
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
        return Err(anyhow!("Target repo is not clean. Please clean or use \"git add\" to add all changes before continue."));
    }

    if status.added.is_empty() {
        // remove temp dir
        path::remove_path(template_apply_dir)?;
        return Err(anyhow!("Nothing is added, so we abort this apply process"));
    }

    // rewrite delta file
    let template_apply_dir = &target_dir.join(".git/gut/template_apply/");
    let new_delta = TargetDelta::get(&template_apply_dir.join("temp_target_delta.toml"))?;
    let delta_path = &target_dir.join(".gut/delta.toml").to_path_buf();
    new_delta.save(&delta_path)?;
    let mut index = target_repo.index()?;
    index.add_path(Path::new(".gut/delta.toml"))?;
    let message = format!("Apply changes {:?}", new_delta.rev_id);

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

fn start_apply(template_dir: &PathBuf, template_delta: &TemplateDelta, target_dir: &PathBuf, optional: bool) -> Result<()> {

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
    File::create(&apply_status_path)?;

    let template_repo = git::open::open(template_dir)?;

    // TODO need to figure out real commit_sha
    let temp_current_sha = "cf64bdc4eebfcaba6393ca2a4b7f49bfacf32016";
    let temp_last_sha = &target_delta.template_sha;

    let generate_files = template_delta.generate_files(optional);
    let diff = git::diff::diff_trees(&template_repo, temp_last_sha, temp_current_sha)?;

    let patch_files = diff_to_patch(&diff)?;

    //for p in &patch_files {
        //println!("======================");
        //println!("{:?}", p);
    //}

    let patch_files: Vec<_> = patch_files.into_iter().filter(|p| generate_files.contains(&p.new_file)).collect();

    let target_patch_files: Vec<_> = patch_files.iter().map(|p| p.apply_patterns(&target_delta.replacements)).collect();
    let target_patch_files: Result<Vec<_>> = target_patch_files.into_iter().collect();

    let diff_path = &template_apply_dir.join("patch.diff");
    write(&diff_path, to_content(&target_patch_files?))?;
    execute_patch(diff_path.to_str().unwrap(), target_dir)?;

    let update_target_delta = target_delta.update(template_delta.rev_id, temp_current_sha);
    update_target_delta.save(&template_apply_dir.join("temp_target_delta.toml"))?;

    Ok(())
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
