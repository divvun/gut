use super::models::*;
use crate::commands::common;
use crate::commands::models::ExistDirectory;
use crate::commands::models::Script;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github::RemoteRepo;
use crate::user::User;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// export data file for ci generate command
pub struct ExportArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub template: ExistDirectory,
    #[structopt(long)]
    pub output: String,
    /// The script that produces name, version and human_name
    pub script: Script,
    /// use https to clone repositories if needed
    #[structopt(long, short)]
    pub use_https: bool,
}

impl ExportArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user.token,
        )?;

        let repos: Result<HashMap<String, RepoData>> = filtered_repos
            .iter()
            .map(|r| get_repo_data(&r, &self.script, &user, self.use_https))
            .collect();

        let repos = repos?;

        match save(&repos, &Path::new(&self.output).to_path_buf()) {
            Ok(_) => println!("Save repos data successfully at {:?}", self.output),
            Err(e) => println!("Failed to export data because {:?}", e),
        }
        Ok(())
    }
}

fn get_repo_data(
    repo: &RemoteRepo,
    script: &Script,
    user: &User,
    use_https: bool,
) -> Result<(String, RepoData)> {
    let git_repo = try_from_one(repo.clone(), user, use_https)?;

    let _cloned_repo = git_repo.open_or_clone()?;
    let data =
        script.execute_and_get_output_with_dir(&git_repo.local_path, &repo.name, &repo.owner)?;

    let p: Input = serde_json::from_str(&data)?;

    let mut package: HashMap<String, String> = HashMap::new();
    package.insert("__NAME__".to_string(), p.name.clone());
    package.insert("__HUMAN_NAME__".to_string(), p.human_name.clone());
    package.insert("__VERSION__".to_string(), p.version.clone());
    package.insert("__TAG__".to_string(), p.language_tag.clone());

    let repo_data = RepoData { package };

    Ok((repo.name.to_string(), repo_data))
}

#[derive(Deserialize)]
struct Input {
    name: String,
    version: String,
    human_name: String,
    language_tag: String,
}
