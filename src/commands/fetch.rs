use super::common::{self, OrgResult};
use crate::cli::Args as CommonArgs;
use crate::filter::Filter;
use crate::git;
use crate::git::GitCredential;
use crate::path;
use crate::user::User;
use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Fetch all local repositories that match a regex
///
/// This command only works on those repositories that has been cloned in root directory
pub struct FetchArgs {
    #[arg(long, short, conflicts_with = "all_orgs")]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Run command against all organizations, not just the default one
    pub all_orgs: bool,
}

impl FetchArgs {
    pub fn run(&self, _common_args: &CommonArgs) -> Result<()> {
        common::run_for_orgs(
            self.all_orgs,
            self.organisation.as_deref(),
            |org| self.run_for_organization(org),
            "Fetched",
        )
    }

    fn run_for_organization(&self, organisation: &str) -> Result<OrgResult> {
        let user = common::user()?;
        let root = common::root()?;

        let sub_dirs = common::read_dirs_for_org(organisation, &root, self.regex.as_ref())?;

        if sub_dirs.is_empty() {
            println!(
                "There is no local repositories in organisation {} matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(OrgResult::new(organisation));
        }

        // Collect results as (repo_name, Result)
        let results: Vec<_> = sub_dirs
            .par_iter()
            .map(|d| {
                let name = path::dir_name(d).unwrap_or_default();
                println!("Fetching {}...", name);
                let result = fetch(d, &user);
                (name, result)
            })
            .collect();

        // Collect errors
        let errors: Vec<_> = results
            .iter()
            .filter_map(|(name, r)| r.as_ref().err().map(|e| (name.as_str(), e)))
            .collect();

        let successful = results.len() - errors.len();
        let failed = errors.len();

        // Print summary
        if errors.is_empty() {
            println!("\nSuccessfully fetched {} repos!", successful);
        } else {
            println!("\nFetched {} repos with {} errors:", successful, failed);
            for (name, err) in &errors {
                println!("  {} - {}", name, err);
            }
        }

        Ok(OrgResult {
            org_name: organisation.to_string(),
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
