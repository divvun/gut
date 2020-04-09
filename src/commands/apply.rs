use super::common;
use super::models::Script;
use crate::filter::Filter;
use crate::path::local_path_org;
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub script: Script,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        let script_path = self
            .script
            .path
            .to_str()
            .expect("dadmin only supports utf8 path now!");
        for dir in sub_dirs {
            match apply(&dir, script_path) {
                Ok(_) => println!(
                    "Applied script {} for dir {:?} successfully",
                    script_path, dir
                ),
                Err(e) => println!(
                    "Failed to apply script {} for dir {:?} because {:?}",
                    script_path, dir, e
                ),
            }
        }

        Ok(())
    }
}

fn apply(dir: &PathBuf, script: &str) -> Result<()> {
    executeScript(script, dir);
    Ok(())
}

fn executeScript(script: &str, dir: &PathBuf) -> Result<()> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", script])
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

    println!("Script result {:?}", output);
    Ok(())
}
