use super::common;
use super::models::Script;
use crate::github;

use crate::github::RemoteRepo;
use anyhow::{anyhow, Result};

use crate::filter::Filter;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set description and/or website for all repositories that match regex
///
/// Description can be provided by --description option or --des-script option
///
/// When it is provided --des-script will override --description
///
/// Similar to --web-script and --website
pub struct InfoArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
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
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos = common::query_and_filter_repositories(
            &organisation,
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
    let des = get_text(
        repo,
        args.description.as_deref(),
        args.des_script.as_ref(),
        "No description is provided",
    );
    let web = get_text(
        repo,
        args.website.as_deref(),
        args.web_script.as_ref(),
        "No website is provided",
    );

    github::set_repo_metadata(&repo, des.ok().as_deref(), web.ok().as_deref(), &token)?;
    Ok(())
}

fn get_text(
    repo: &RemoteRepo,
    op_text: Option<&str>,
    op_script: Option<&Script>,
    err_msg: &str,
) -> Result<String> {
    println!("get_text {:?}: {:?}", op_text, op_script);
    if let Some(script) = op_script {
        script.execute_and_get_output(&repo.name, &repo.owner)
    } else {
        op_text
            .ok_or_else(|| anyhow!("{}", err_msg))
            .map(|s| s.to_string())
    }
}
