use super::models::*;
use crate::cli::Args as CommonArgs;
use crate::commands::common;
use crate::commands::models::ExistDirectory;
use crate::commands::models::Script;
use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github::RemoteRepo;
use crate::user::User;
use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Parser)]
/// export data file for ci generate command
pub struct ExportArgs {
    #[arg(long, short, default_value = "divvun")]
    pub organisation: String,
    #[arg(long, short, required_unless_present("topic"))]
    pub regex: Option<Filter>,
    #[arg(long, required_unless_present("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[arg(long, short)]
    pub template: ExistDirectory,
    #[arg(long)]
    pub output: String,
    /// The script that produces name, version and human_name
    #[arg(long, short)]
    pub script: Script,
    /// use https to clone repositories if needed
    #[arg(long, short)]
    pub use_https: bool,
}

impl ExportArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;
        let filtered_repos =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref());

        let repos: Result<BTreeMap<String, RepoData>> = filtered_repos
            .iter()
            .map(|r| get_repo_data(&r.repo, &self.script, &user, self.use_https))
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

    let count = &p.version.split('.').count();
    let version = if *count == 2 {
        format!("{}.0", &p.version)
    } else {
        p.version.clone()
    };

    let mut package: BTreeMap<String, String> = BTreeMap::new();
    package.insert("__NAME__".to_string(), p.name.clone());
    package.insert("__HUMAN_NAME__".to_string(), p.human_name.clone());
    package.insert("__VERSION__".to_string(), version);
    package.insert("__TAG__".to_string(), p.language_tag);

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
