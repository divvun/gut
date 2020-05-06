use super::common::*;
use super::model::*;
use crate::commands::common;
use crate::commands::models::ExistDirectory;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use git2::Repository;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use std::str;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GenerateArgs {
    #[structopt(long, short)]
    pub template: ExistDirectory,
    #[structopt(long, short)]
    pub dir: String,
    #[structopt(long)]
    pub optional: bool,
}

impl GenerateArgs {
    pub fn run(&self) -> Result<()> {
        let template_dir = &self.template.path;
        let target_dir = Path::new(&self.dir).to_path_buf();
        create_dir_all(&target_dir).context("Cannot create target directory")?;

        match generate(&template_dir, &target_dir, self.optional) {
            Ok(_) => println!("Generate success"),
            Err(e) => println!("Generate failed {:?}", e),
        }
        Ok(())
    }
}

// generate content
// init git repo
// create delta files
// commit all
fn generate(template_dir: &PathBuf, target_dir: &PathBuf, optional: bool) -> Result<()> {
    let template_repo = git::open(template_dir)?;
    let current_sha = git::head_sha(&template_repo)?;

    let template_delta = TemplateDelta::get(&template_dir.join(".gut/template.toml"))?;
    let target_info = get_target_info(&template_delta)?;

    // generate file paths
    let generate_files = template_delta.generate_files(optional);
    let rx = generate_files.iter().map(AsRef::as_ref).collect();
    let target_files = generate_file_paths(&target_info.reps, rx)?;
    println!("Target files {:?}", target_files);

    // wirte content
    for (original, target) in target_files {
        let original_path = template_dir.join(&original);
        let target_path = target_dir.join(&target);
        let original_content = read_to_string(&original_path)?;
        let target_content = generate_string(&target_info.reps, original_content.as_str())?;
        println!("generated content for {:?}", target_path);
        println!("{}", target_content);
        println!("");
        write_content(&target_path, &target_content)?;
    }

    // init repo
    let target_repo = Repository::init(target_dir)?;

    // write delta file
    let target_delta = TargetDelta {
        template: "".to_string(),
        rev_id: template_delta.rev_id,
        template_sha: current_sha,
        replacements: target_info.reps.clone(),
    };
    let gut_path = &target_dir.join(".gut/").to_path_buf();
    create_dir_all(&gut_path)?;
    target_delta.save(&gut_path.join("delta.toml").to_path_buf())?;

    // commit all data
    commit(&target_repo, "Generate project")?;
    Ok(())
}

fn write_content(file_path: &PathBuf, content: &str) -> Result<()> {
    let parrent = path::parrent(file_path)?;
    create_dir_all(&parrent)?;
    write(file_path, content)?;
    Ok(())
}

struct TargetInfo {
    name: String,
    reps: HashMap<String, String>,
}

fn get_target_info(template_delta: &TemplateDelta) -> Result<TargetInfo> {
    let name = common::ask_for("Project name")?;
    println!("Enter patterns:");
    let mut reps = HashMap::new();
    for pattern in &template_delta.patterns {
        let key = common::ask_for(pattern)?;
        reps.insert(pattern.to_string(), key);
    }

    Ok(TargetInfo { name, reps })
}

pub fn commit(git_repo: &Repository, msg: &str) -> Result<()> {
    let status = git::status(&git_repo, true)?;

    let mut index = git_repo.index()?;

    let addable_list = status.addable_list();
    for p in addable_list {
        //log::debug!("addable file: {}", p);
        let path = Path::new(&p);
        index.add_path(path)?;
    }

    for p in status.deleted {
        //log::debug!("removed file: {}", p);
        let path = Path::new(&p);
        index.remove_path(path)?;
    }

    git::commit_first(&git_repo, &mut index, msg)?;

    Ok(())
}
