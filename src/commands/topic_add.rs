use super::common;
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::github;
use anyhow::Result;
use clap::Parser;
use prettytable::{Table, format, row};
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Add topics for all repositories that match a regex
pub struct TopicAddArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// All topics will be added
    pub topics: Vec<String>,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl TopicAddArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        if self.all_orgs {
            let organizations = common::get_all_organizations()?;
            if organizations.is_empty() {
                println!("No organizations found in root directory");
                return Ok(());
            }
            
            let mut summaries = Vec::new();
            
            for org in &organizations {
                println!("\n=== Processing organization: {} ===", org);
                
                match self.run_for_organization(org) {
                    Ok(summary) => {
                        summaries.push(summary);
                    },
                    Err(e) => {
                        println!("Failed to process organization '{}': {:?}", org, e);
                    }
                }
            }
            
            print_topic_add_summary(&summaries);
            
            Ok(())
        } else {
            let organisation = common::organisation(self.organisation.as_deref())?;
            self.run_for_organization(&organisation)?;
            Ok(())
        }
    }

    fn run_for_organization(&self, organisation: &str) -> Result<common::OrgResult> {
        let user_token = common::user_token()?;

        let filtered_repos =
            common::query_and_filter_repositories(organisation, self.regex.as_ref(), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in organisation {} that match the pattern {:?}",
                organisation, self.regex
            );
            return Ok(common::OrgResult::new(organisation.to_string()));
        }

        let results: Vec<_> = filtered_repos.par_iter().map(|repo| {
            let result = add_topics(repo, &self.topics, &user_token);
            match result {
                Ok(topics) => {
                    println!("Add topics for repo {} successfully", repo.name);
                    println!("List of topics for {} is: {:?}", repo.name, topics);
                    true
                }
                Err(e) => {
                    println!(
                        "Failed to add topics for repo {} because {:?}",
                        repo.name, e
                    );
                    false
                }
            }
        }).collect();

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

fn add_topics(repo: &github::RemoteRepo, topics: &[String], token: &str) -> Result<Vec<String>> {
    let current_topics = github::get_topics(repo, token)?;
    let temp = vec![current_topics, topics.to_owned()];

    let new_topics: Vec<String> = temp.into_iter().flatten().collect();

    github::set_topics(repo, &new_topics, token)
}

fn print_topic_add_summary(summaries: &[common::OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Organisation", "#repos", "Topics Added", "Failed"]);

    for summary in summaries {
        table.add_row(row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
    }

    println!("\n=== All org summary ===");
    table.printstd();
}
