use super::models::*;
use crate::cli::Args as CommonArgs;
use crate::commands::common;
use crate::commands::models::template::*;
use crate::commands::models::ExistDirectory;
use crate::commands::patterns::*;
use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github::RemoteRepo;
use crate::path;
use crate::user::User;
use anyhow::Result;
use clap::Parser;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Parser)]
/// generate ci for every repositories that matches
pub struct GenerateArgs {
    #[arg(long, short, default_value = "divvun")]
    pub organisation: String,
    #[arg(long, short)]
    pub regex: Option<Filter>,
    #[arg(long, required_unless_present("regex"))]
    /// topic to filter
    pub topic: Option<String>,
    #[arg(long, short)]
    pub template: ExistDirectory,
    #[arg(long, short)]
    pub data: String,
    /// use https to clone repositories if needed
    #[arg(long, short)]
    pub use_https: bool,
}

impl GenerateArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;
        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        let data = get(&Path::new(&self.data).to_path_buf())?;

        for repo in filtered_repos {
            match data.get(&repo.name) {
                Some(repo_data) => {
                    match generate_ci(&repo, &self.template.path, repo_data, &user, self.use_https)
                    {
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
    template_dir: &Path,
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
    let content = read_to_string(manifest_path)?;
    let target_content = generate_string(&data.package, &content)?;
    let target_content = generate_uuids(&target_content);
    Ok(target_content)
}

pub fn generate_uuids(content: &str) -> String {
    let mut result = String::new();
    let from = "__UUID__";

    let mut last_end = 0;
    for (start, part) in content.match_indices(from) {
        result.push_str(unsafe { content.get_unchecked(last_end..start) });
        let uuid = Uuid::new_v4();
        // TODO make them capitalized
        result.push_str(&uuid.to_string().to_uppercase());
        last_end = start + part.len();
    }
    result.push_str(unsafe { content.get_unchecked(last_end..content.len()) });

    result
}
