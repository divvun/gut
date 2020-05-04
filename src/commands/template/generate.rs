use std::collections::HashMap;
use crate::commands::common;
use std::str;
use crate::path;
use std::path::{Path, PathBuf};
use super::model::*;
use super::common::*;
use anyhow::{Context, Result};
use structopt::StructOpt;
use std::fs::{read_to_string, write, create_dir_all};
use crate::commands::models::ExistDirectory;

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
        println!("Template apply args {:?}", self);

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

fn generate(template_dir: &PathBuf, target_dir: &PathBuf, optional: bool) -> Result<()> {
    let template_delta = temp_sample();
    let target_info = get_target_info(&template_delta)?;

    // generate file paths
    let generate_files = template_delta.generate_files(optional);
    let rx = generate_files.iter().map(AsRef::as_ref).collect();
    let targetd_files = generate_file_paths(&target_info.reps, rx)?;
    println!("Target files {:?}", targetd_files);

    for (original, target) in targetd_files {
        let original_path = template_dir.join(&original);
        let target_path = target_dir.join(&target);
        let original_content = read_to_string(&original_path)?;
        let target_content = generate_string(&target_info.reps, original_content.as_str())?;
        println!("generated content for {:?}",target_path);
        println!("{}", target_content);
        println!("");
        write_content(&target_path, &target_content)?;
    }

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

    Ok(TargetInfo {
        name,
        reps,
    })
}
