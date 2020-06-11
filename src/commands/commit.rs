use super::common;
use crate::filter::Filter;
use crate::git;
use anyhow::Result;
use std::path::Path;
use structopt::StructOpt;

use crate::commands::topic_helper;
use crate::convert::try_from_one;
use crate::github::RemoteRepo;
use crate::user::User;

#[derive(Debug, StructOpt)]
/// Add all and then commit with the provided messages for all
/// repositories that match a pattern or a topic
pub struct CommitArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    /// topic to filter
    pub topic: Option<String>,
    #[structopt(long, short)]
    /// Commit message
    pub message: String,
    #[structopt(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
}

impl CommitArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;

        let all_repos =
            topic_helper::query_repositories_with_topics(&self.organisation, &user.token)?;
        let filtered_repos: Vec<_> =
            topic_helper::filter_repos(&all_repos, self.topic.as_ref(), self.regex.as_ref())
                .into_iter()
                .map(|r| r.repo)
                .collect();

        for repo in filtered_repos {
            let dir_name = &repo.name;
            match commit(&repo, &self.message, &user, self.use_https) {
                Err(e) => println!("{}: Failed to commit because {:?}", dir_name, e),
                Ok(result) => match result {
                    CommitResult::Conflict => println!(
                        "{}: There are conflicts. Fix conflicts and then commit the results.",
                        dir_name
                    ),
                    CommitResult::NoChanges => println!("{}: There is no changes.", dir_name),
                    CommitResult::Success => println!("{}: Commit success.", dir_name),
                },
            }
        }
        Ok(())
    }
}

pub fn commit(repo: &RemoteRepo, msg: &str, user: &User, use_https: bool) -> Result<CommitResult> {
    let git_repo = try_from_one(repo.clone(), user, use_https)?;
    let git_repo = git_repo.open()?;

    let status = git::status(&git_repo, true)?;
    //let current_branch = git::head_shorthand(&git_repo)?;

    if !status.can_commit() {
        return Ok(CommitResult::Conflict);
    }

    if !status.should_commit() {
        return Ok(CommitResult::NoChanges);
    }

    let mut index = git_repo.index()?;

    let addable_list = status.addable_list();
    for p in addable_list {
        //log::debug!("addable file: {}", p);
        let path = Path::new(&p);
        index.add_path(path)?;
    }

    for p in status.deleted {
        //log::debug!("removed file: {}", p);
        let path = Path::new(&p);
        index.remove_path(path)?;
    }

    git::commit_index(&git_repo, &mut index, msg)?;

    Ok(CommitResult::Success)
}

pub enum CommitResult {
    Conflict,
    NoChanges,
    Success,
}
