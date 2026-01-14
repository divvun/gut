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
use prettytable::{Table, format, row};
use rayon::prelude::*;
use std::process::Output;

/// Apply a script to all repositories that has a topics that match a pattern
/// Or to all repositories that has a specific topic
#[derive(Debug, Parser)]
pub struct TopicApplyArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
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
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl TopicApplyArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        common::run_for_orgs_or_single(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org),
            Some(print_topic_apply_summary),
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<common::OrgResult> {
        println!("Topic apply for organization: {}", organisation);

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        let user = common::user()?;

        let repos = topic_helper::query_repositories_with_topics(organisation, &user.token)?;
        let repos =
            topic_helper::filter_repos_by_topics(&repos, self.topic.as_ref(), self.regex.as_ref());

        println!("repos {:?}", repos);

        let results: Vec<_> = repos
            .par_iter()
            .map(
                |repo| match apply(repo, script_path, &user, self.use_https) {
                    Ok(_) => {
                        println!("Apply success for repo {}", repo.repo.name);
                        true
                    }
                    Err(e) => {
                        println!("Apply failed for repo {} because {:?}", repo.repo.name, e);
                        false
                    }
                },
            )
            .collect();

        let successful = results.iter().filter(|&&success| success).count();
        let failed = results.len() - successful;

        Ok(common::OrgResult {
            org_name: organisation.to_string(),
            total_repos: results.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
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

fn print_topic_apply_summary(summaries: &[common::OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Topics Applied", "Failed"]);

    let mut total_repos = 0;
    let mut total_applied = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_applied += summary.successful_repos;
        total_failed += summary.failed_repos;
    }
    table.add_empty_row();
    table.add_row(row![
        "TOTAL",
        r -> total_repos,
        r -> total_applied,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
