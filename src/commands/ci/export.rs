use super::models::*;
use crate::commands::common;
use crate::commands::models::template::*;
use crate::commands::models::ExistDirectory;
use crate::filter::Filter;
use crate::github::RemoteRepo;
use anyhow::Result;
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

    #[structopt(long, default_value = "repos_data.toml")]
    pub output: String,
}

impl ExportArgs {
    pub fn run(&self) -> Result<()> {
        let template_delta = TemplateDelta::get(&self.template.path.join(".gut/template.toml"))?;
        let user = common::user()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user.token,
        )?;

        // get patterns from template project
        let patterns = &template_delta.patterns;
        let repos: HashMap<String, RepoData> = filtered_repos
            .iter()
            .map(|r| get_repo_data(&r, patterns))
            .collect();
        match save(&repos, &Path::new(&self.output).to_path_buf()) {
            Ok(_) => println!("Save repos data successfully at {:?}", self.output),
            Err(e) => println!("Failed to export data because {:?}", e),
        }
        Ok(())
    }
}

fn get_repo_data(repo: &RemoteRepo, patterns: &[String]) -> (String, RepoData) {
    let todo = "TODO";
    let mut file_map: HashMap<String, Speller> = HashMap::new();
    file_map.insert(
        todo.to_string(),
        Speller {
            filename: todo.to_string(),
            name_win: Some(todo.to_string()),
        },
    );
    file_map.insert(
        "TODO 1".to_string(),
        Speller {
            filename: todo.to_string(),
            name_win: None,
        },
    );

    let mut package: HashMap<String, String> = HashMap::new();
    for p in patterns {
        if p == "name" {
            package.insert("name".to_string(), repo.name.to_string());
        } else {
            package.insert(p.clone(), todo.to_string());
        }
    }
    let repo_data = RepoData {
        package,
        spellers: file_map,
    };
    (repo.name.to_string(), repo_data)
}
