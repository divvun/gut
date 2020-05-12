use super::models::*;
use crate::commands::common;
use crate::commands::models::template::*;
use crate::commands::models::ExistDirectory;
use crate::commands::patterns::*;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github::RemoteRepo;
use crate::path;
use crate::user::User;
use anyhow::Result;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
/// export data file for ci generate command
pub struct GenerateArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    pub template: ExistDirectory,
    #[structopt(long, short)]
    pub data: String,
    /// use https to clone repositories if needed
    #[structopt(long, short)]
    pub use_https: bool,
}

impl GenerateArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user.token,
        )?;

        let data = get(&Path::new(&self.data).to_path_buf())?;

        for repo in filtered_repos {
            match data.get(&repo.name) {
                Some(repo_data) => {
                    match generate_ci(
                        &repo,
                        &self.template.path,
                        &repo_data,
                        &user,
                        self.use_https,
                    ) {
                        Ok(_) => println!("Generate ci successfully for {:?}", repo.name),
                        Err(e) => {
                            println!("Failed to generate ci for {:?} because {:?}", repo.name, e)
                        }
                    }
                }
                None => println!("There is no data for {:?}", repo.name),
            }
        }

        Ok(())
    }
}

/// process manifest toml with patterns
/// generate uuid
/// write to file
fn generate_ci(
    repo: &RemoteRepo,
    template_dir: &PathBuf,
    data: &RepoData,
    user: &User,
    use_https: bool,
) -> Result<()> {
    let template_delta = TemplateDelta::get(&template_dir.join(".gut/template.toml"))?;

    let git_repo = try_from_one(repo.clone(), user, use_https)?;

    let cloned_repo = git_repo.open_or_clone()?;
    log::debug!("Cloned repo: {:?}", cloned_repo.path());

    // generate file paths
    let files_to_generate = template_delta.generate_files(false);

    let manifest_path = ".gut/manifest.toml";

    for file in &files_to_generate {
        let original_path = template_dir.join(file);
        let content = if file == manifest_path {
            process_manifest(&original_path, data)?
        } else {
            read_to_string(&original_path)?
        };
        let target_path = git_repo.local_path.join(file);
        path::write_content(&target_path, &content)?;
    }

    Ok(())
}

/// replace with patterns
/// generate uuids
/// set spellers
fn process_manifest(manifest_path: &PathBuf, data: &RepoData) -> Result<String> {
    let content = read_to_string(&manifest_path)?;
    let target_content = generate_string(&data.package, &content)?;
    let target_content = generate_uuids(&target_content);
    let manifest = Manifest::get_from_content(&target_content)?;
    let spellers = data.spellers.clone();
    let manifest = manifest.set_spellers(&spellers);
    let result = manifest.to_content()?;
    Ok(result)
}

pub fn generate_uuids(content: &str) -> String {
    let mut result = String::new();
    let from = "__UUID__";

    let mut last_end = 0;
    for (start, part) in content.match_indices(from) {
        result.push_str(unsafe { content.get_unchecked(last_end..start) });
        let uuid = Uuid::new_v4();
        result.push_str(&uuid.to_string());
        last_end = start + part.len();
    }
    result.push_str(unsafe { content.get_unchecked(last_end..content.len()) });

    result
}
