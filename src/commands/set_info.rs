use super::common;
use super::models::Script;
use crate::github;

use crate::github::RemoteRepo;
use anyhow::{anyhow, Result};
use std::process::{Command, Output};
use std::str;

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set description and/or website for all repositories that match regex
/// Description can be provided by --description option or --des-script option
/// --des-script will override --description if it is provided
/// Similar to --web-script and --website
/// The script can use two arguments
/// repository name as argument number one
/// organisation name as argument number two
///
/// Here is a sample of Description scrip
/// ```
/// name=$1
/// org=$2
/// printf "This is the best description ever for ${name} in ${org}"
/// ```
pub struct InfoArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Filter,
    #[structopt(long, short)]
    /// Description, this is required unless website is provided
    pub description: Option<String>,
    #[structopt(long, short)]
    /// Homepage, this is required unless description is provided
    pub website: Option<String>,
    #[structopt(long)]
    /// The script that will produce a description
    pub des_script: Option<Script>,
    #[structopt(long)]
    /// The script that will produce a website
    pub web_script: Option<Script>,
}

impl InfoArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            Some(&self.regex),
            &user_token,
        )?;

        for repo in filtered_repos {
            let result = set_info(&repo, &self, &user_token);
            match result {
                Ok(_) => println!("Set info for repo {} successfully", repo.name),
                Err(e) => println!("Failed to set info for repo {} because {:?}", repo.name, e),
            }
        }
        Ok(())
    }
}

fn set_info(repo: &RemoteRepo, args: &InfoArgs, token: &str) -> Result<()> {
    let des = get_text(repo, args.description.as_deref(), args.des_script.as_ref());
    let web = get_text(repo, args.website.as_deref(), args.web_script.as_ref());

    github::set_repo_metadata(&repo, des.ok().as_deref(), web.ok().as_deref(), &token)?;
    Ok(())
}

fn get_text(
    repo: &RemoteRepo,
    op_text: Option<&str>,
    op_script: Option<&Script>,
) -> Result<String> {
    println!("get_text {:?}: {:?}", op_text, op_script);
    if let Some(script) = op_script {
        let script_path = script
            .path
            .to_str()
            .expect("dadmin only supports utf8 path now!");

        get_text_from_script(script_path, &repo.name, &repo.owner)
    } else {
        op_text
            .ok_or(anyhow!("No description is provided"))
            .map(|s| s.to_string())
    }
}

fn get_text_from_script(script: &str, name: &str, org: &str) -> Result<String> {
    let output = execute_script(script, name, org)?;
    if output.status.success() {
        let stdout = str::from_utf8(&output.stdout)?;
        log::info!("Out put of the script: {}", stdout);
        Ok(stdout.to_string())
    } else {
        let err_message = String::from_utf8(output.stderr)
            .unwrap_or_else(|_| format!("Cannot execute the script {}", script));
        Err(anyhow!(err_message))
    }
}

fn execute_script(script: &str, name: &str, org: &str) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", script])
            .arg(name)
            .arg(org)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg(script)
            .arg(name)
            .arg(org)
            .output()
            .expect("failed to execute process")
    };

    log::debug!("Script result {:?}", output);

    Ok(output)
}
