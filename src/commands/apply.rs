use super::common;
use super::models::Script;
use crate::filter::Filter;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::{Command, Output};
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
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

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
    let output = execute_script(script, dir)?;
    if output.status.success() {
        Ok(())
    } else {
        let err_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| format!("Cannot execute the script {}", script));
        Err(anyhow!(err_message))
    }
}

fn execute_script(script: &str, dir: &PathBuf) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", script])
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

    log::debug!("Script result {:?}", output);

    Ok(output)
}
