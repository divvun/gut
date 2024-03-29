use crate::cli::Args as CommonArgs;
use crate::commands::common;
use crate::commands::models::template::*;
use crate::commands::models::ExistDirectory;
use crate::commands::patterns::*;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use git2::{Repository, RepositoryInitOptions};
use std::collections::BTreeMap;
use std::fs::{copy, create_dir_all, read_to_string};
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug, Parser)]
/// Generate a new project from a template
pub struct GenerateArgs {
    /// Directory of template project
    #[arg(long, short)]
    pub template: ExistDirectory,
    /// Directory of the will be genrated project
    #[arg(long, short)]
    pub dir: String,
    /// Option to skip git init for new project
    #[arg(long, short)]
    pub no_init: bool,
}

impl GenerateArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let template_dir = &self.template.path;
        let target_dir = Path::new(&self.dir).to_path_buf();
        create_dir_all(&target_dir).context("Cannot create target directory")?;

        match generate(template_dir, &target_dir, self.no_init) {
            Ok(_) => println!("Generate success at {:?}", target_dir),
            Err(e) => println!("Generate failed because {:?}", e),
        }
        Ok(())
    }
}

// generate content
// init git repo
// create delta files
// commit all
fn generate(template_dir: &PathBuf, target_dir: &PathBuf, no_init: bool) -> Result<()> {
    let template_repo = git::open(template_dir)?;
    let current_sha = git::head_sha(&template_repo)?;

    let template_delta = TemplateDelta::get(&template_dir.join(".gut/template.toml"))?;
    let target_info = get_target_info(&template_delta)?;

    // generate file paths
    let generate_files = path::all_files(template_dir);
    let rx = generate_files.iter().map(AsRef::as_ref).collect();
    let target_files = generate_file_paths(&target_info.reps, rx)?;

    // wirte content
    for (original, target) in target_files {
        let original_path = template_dir.join(&original);
        let target_path = target_dir.join(&target);
        if let Ok(original_content) = read_to_string(&original_path) {
            let target_content = generate_string(&target_info.reps, original_content.as_str())?;
            path::write_content(&target_path, &target_content)?;
        } else {
            let parrent = path::parrent(&target_path)?;
            create_dir_all(&parrent)?;
            copy(original_path, target_path)?;
        }
    }

    // write delta file
    let target_delta = TargetDelta {
        template: "".to_string(),
        rev_id: template_delta.rev_id,
        template_sha: current_sha,
        replacements: target_info.reps,
    };
    let gut_path = &target_dir.join(".gut/");
    create_dir_all(gut_path)?;
    target_delta.save(&gut_path.join("delta.toml"))?;

    if !no_init {
        // init repo
        let mut rio = RepositoryInitOptions::new();
        rio.initial_head("main");
        let target_repo = Repository::init_opts(target_dir, &rio)?;
        // commit all data
        commit(&target_repo, "Generate project")?;
    }
    Ok(())
}

struct TargetInfo {
    reps: BTreeMap<String, String>,
}

fn get_target_info(template_delta: &TemplateDelta) -> Result<TargetInfo> {
    println!("Enter patterns:");
    let mut reps = BTreeMap::new();
    for pattern in &template_delta.patterns {
        let key = common::ask_for(pattern)?;
        reps.insert(pattern.to_string(), key);
    }

    Ok(TargetInfo { reps })
}

pub fn commit(git_repo: &Repository, msg: &str) -> Result<()> {
    let status = git::status(git_repo, true)?;

    let mut index = git_repo.index()?;

    let addable_list = status.addable_list();
    for p in addable_list {
        log::debug!("addable file: {}", p);
        let path = Path::new(&p);
        index.add_path(path)?;
    }

    for p in status.deleted {
        log::debug!("removed file: {}", p);
        let path = Path::new(&p);
        index.remove_path(path)?;
    }

    git::commit_first(git_repo, &mut index, msg)?;

    Ok(())
}
