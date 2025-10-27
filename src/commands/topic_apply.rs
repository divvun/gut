use super::common;
use super::models::Script;
use super::topic_helper;
use crate::cli::Args as CommonArgs;
use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::github::RemoteRepoWithTopics;
use crate::user::User;
use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use std::process::Output;

/// Apply a script to all repositories that has a topics that match a pattern
/// Or to all repositories that has a specific topic
#[derive(Debug, Parser)]
pub struct TopicApplyArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    /// regex pattern to filter topics. This is required unless topic is provided.
    #[arg(long, short, required_unless_present("topic"))]
    pub regex: Option<Filter>,
    /// A topic to filter repositories. This is required unless regex is provided.
    #[arg(long, short, required_unless_present("regex"))]
    pub topic: Option<String>,
    /// The script will be applied for all repositories that match
    #[arg(long, short)]
    pub script: Script,
    /// use https to clone repositories if needed
    #[arg(long, short)]
    pub use_https: bool,
}

impl TopicApplyArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        println!("Topic apply {:?}", self);

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        let user = common::user()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let repos = topic_helper::query_repositories_with_topics(&organisation, &user.token)?;
        let repos =
            topic_helper::filter_repos_by_topics(&repos, self.topic.as_ref(), self.regex.as_ref());

        println!("repos {:?}", repos);

        repos.par_iter().for_each(|repo| {
            match apply(repo, script_path, &user, self.use_https) {
                Ok(_) => println!("Apply success"),
                Err(e) => println!("Apply failed because {:?}", e),
            }
        });

        Ok(())
    }
}

fn apply(
    repo: &RemoteRepoWithTopics,
    script_path: &str,
    user: &User,
    use_https: bool,
) -> Result<Output> {
    let git_repo = try_from_one(repo.repo.clone(), user, use_https)?;

    let cloned_repo = git_repo.open_or_clone()?;
    log::debug!("Cloned repo: {:?}", cloned_repo.path());

    common::apply_script(&git_repo.local_path, script_path)
}
