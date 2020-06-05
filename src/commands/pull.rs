use super::common;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::git::MergeStatus;
use crate::path;
use crate::user::User;
use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::{Context, Error, Result};

#[derive(Debug, StructOpt)]
/// Pull the current branch of all local repositories that match a regex
///
/// This command only works on those repositories that has been cloned in root directory
pub struct PullArgs {
    #[structopt(long, short)]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// stash if there are unstaged changes
    pub stash: bool,
}

impl PullArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        for dir in sub_dirs {
            let dir_name = path::dir_name(&dir)?;
            println!("Pulling for {}", dir_name);

            let status = pull(&dir, &user, self.stash);
            println!("status {:?}", status);
        }

        Ok(())
    }
}



fn pull(dir: &PathBuf, user: &User, stash: bool) -> Status {

    let mut dir_name = "".to_string();
    let mut repo_status = RepoStatus::Clean;
    let mut stash_status = StashStatus::No;

    let mut pull = || -> Result<MergeStatus> {

        dir_name = path::dir_name(&dir)?;
        let mut git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

        let status = git::status(&git_repo, false)?;

        if status.is_empty() {
            stash_status = StashStatus::No;
            repo_status = RepoStatus::Clean;
            // pull
            let cred = GitCredential::from(user);
            let status = git::pull(&git_repo, "origin", Some(cred))?;
            Ok(status)
        } else {
            if status.conflicted.is_empty() {
                repo_status = RepoStatus::Dirty;

                if stash {
                    // do stash
                    stash_status = match git::stash(&mut git_repo, None) {
                        Ok(_) => StashStatus::Success,
                        Err(e) => StashStatus::Failed(e),
                    };
                    // pull
                    let cred = GitCredential::from(user);
                    let status = git::pull(&git_repo, "origin", Some(cred))?;
                    return Ok(status);
                }
            } else {
                repo_status = RepoStatus::Conflict;
            }

            stash_status = StashStatus::Skip;
            Ok(MergeStatus::Nothing)
        }
    };

    let status = pull();

    Status {
        repo: dir_name,
        status,
        repo_status,
        stash_status,
    }
}

#[derive(Debug)]
struct Status {
    repo: String,
    status: Result<MergeStatus>,
    repo_status: RepoStatus,
    stash_status: StashStatus,
}

#[derive(Debug)]
enum StashStatus {
    No,
    Skip,
    Success,
    Failed(Error)
}

#[derive(Debug)]
enum RepoStatus {
    Clean,
    Dirty,
    Conflict,
}
