use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::git;
use crate::path;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Do git clean -f for all local repositories that match a pattern
pub struct CleanArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

impl CleanArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Cleaned",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(owner, &root, self.regex.as_ref())?;

        let total_count = sub_dirs.len();
        let mut success_count = 0;
        let mut fail_count = 0;

        for dir in sub_dirs {
            match clean(&dir) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    fail_count += 1;
                    println!("Failed to clean dir {:?} because {:?}", dir, e);
                }
            }
        }

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: total_count,
            successful_repos: success_count,
            failed_repos: fail_count,
            dirty_repos: 0,
        })
    }
}

fn clean(dir: &PathBuf) -> Result<()> {
    println!("Cleaning {:?}", dir);
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;
    let status = git::status(&git_repo, false)?;
    //println!("git status {:?}", status);

    if status.new.is_empty() {
        println!("Nothing to clean!\n");
    } else {
        println!("Files/directories removed: ");
        for f in status.new {
            let rf = dir.join(f);
            path::remove_path(&rf).with_context(|| format!("Cannot remove {:?}", rf))?;
            println!("{:?}", rf);
        }
        println!();
    }

    Ok(())
}
