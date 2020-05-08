use super::common;
use super::models::Script;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::{anyhow, Result};
use std::process::{Command, Output};
use std::str;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct DescriptionArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Filter,
    #[structopt(long, short)]
    pub script: Script,
}

impl DescriptionArgs {
    pub fn run(&self) -> Result<()> {
        let script_path = self
            .script
            .path
            .to_str()
            .expect("dadmin only supports utf8 path now!");

        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        for repo in filtered_repos {
            match set_description(&repo, script_path, &user_token) {
                Ok(des) => println!(
                    "Set description {} for repo {:?} successfully",
                    des, repo.name
                ),
                Err(e) => println!(
                    "Failed to set description repo {:?} because {:?}",
                    repo.name, e
                ),
            }
        }

        Ok(())
    }
}

fn set_description(repo: &RemoteRepo, script: &str, token: &str) -> Result<String> {
    let des = get_description(script, &repo.name)?;
    println!("description {}", des);
    github::set_repo_metadata(&repo, Some(&des), None, &token)?;
    Ok(des)
}

fn get_description(script: &str, name: &str) -> Result<String> {
    let output = execute_script(script, name)?;
    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout)?;
        Ok(stdout.to_string())
    } else {
        let err_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| format!("Cannot execute the script {}", script));
        Err(anyhow!(err_message))
    }
}

fn execute_script(script: &str, name: &str) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", script])
            .arg(name)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg(script)
            .arg(name)
            .output()
            .expect("failed to execute process")
    };

    log::debug!("Script result {:?}", output);

    Ok(output)
}
