use super::common::{self, OrgResult};
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::path;
use crate::user::User;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Fetch all local repositories that match a regex
///
/// This command only works on those repositories that has been cloned in root directory
pub struct FetchArgs {
    #[arg(long, short, alias = "organisation", conflicts_with = "all_owners")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short, alias = "all-orgs")]
    /// Run command against all owners, not just the default one
    pub all_owners: bool,
}

impl FetchArgs {
    pub fn run(&self) -> Result<()> {
        common::run_for_owners(
            self.all_owners,
            self.owner.as_deref(),
            |owner| self.run_for_owner(owner),
            "Fetched",
        )
    }

    fn run_for_owner(&self, owner: &str) -> Result<OrgResult> {
        let user = common::user()?;
        let root = common::root()?;

        let sub_dirs = common::read_dirs_for_org(owner, &root, self.regex.as_ref())?;

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in {} matches pattern {:?}",
                owner, self.regex
            );
            return Ok(OrgResult::new(owner));
        }

        let results = common::process_with_progress(
            "Fetching",
            &sub_dirs,
            |d| {
                let name = path::dir_name(d).unwrap_or_default();
                let result = fetch(d, &user);
                (name, result)
            },
            |(name, _)| name.clone(),
        );

        // Collect errors
        let errors: Vec<_> = results
            .iter()
            .filter_map(|(name, r)| r.as_ref().err().map(|e| (name.as_str(), e)))
            .collect();

        let successful = results.len() - errors.len();
        let failed = errors.len();

        // Print summary
        if errors.is_empty() {
            println!("Successfully fetched {} repos!", successful);
        } else {
            println!("Fetched {} repos with {} errors:", successful, failed);
            for (name, err) in &errors {
                println!("  {} - {}", name, err);
            }
        }

        Ok(OrgResult {
            org_name: owner.to_string(),
            total_repos: sub_dirs.len(),
            successful_repos: successful,
            failed_repos: failed,
            dirty_repos: 0,
        })
    }
}

fn fetch(dir: &PathBuf, user: &User) -> Result<()> {
    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let cred = GitCredential::from(user);
    git::fetch(&git_repo, "origin", Some(cred), true)?;

    Ok(())
}
