use super::common;
use crate::path::local_path_org;
use std::path::PathBuf;
use anyhow::{Result, bail};
use crate::filter::Filter;
use structopt::StructOpt;

use std::process::Command;

#[derive(Debug, StructOpt)]
pub struct ApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub script: String,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let target_dir = local_path_org(&self.organisation)?;

        let sub_dirs = common::read_dirs_with_option(&target_dir, &self.regex)?;

        for dir in sub_dirs {
            match apply(&dir, &self.script) {
                Ok(_) => println!("Applied script {} for dir {:?} successfully", self.script, dir),
                Err(e) => println!("Failed to apply script {} for dir {:?} because {:?}", &self.script, dir, e),
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
