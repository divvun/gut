use super::common::{self, OrgResult};
use crate::cli::Args as CommonArgs;
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
            Some(print_fetch_summary),
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
            return Ok(OrgResult::new(organisation.to_string()));
        }

        let mut successful = 0;
        let mut failed = 0;

        for dir in &sub_dirs {
            match fetch(dir, &user) {
                Ok(_) => successful += 1,
                Err(e) => {
                    println!("Error fetching: {:?}", e);
                    failed += 1;
                }
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
    let dir_name = path::dir_name(dir)?;
    println!("Fetching for {}", dir_name);

    let git_repo = git::open(dir).with_context(|| format!("{:?} is not a git directory.", dir))?;

    let cred = GitCredential::from(user);
    git::fetch(&git_repo, "origin", Some(cred))?;

    println!("===============");
    Ok(())
}

fn print_fetch_summary(summaries: &[OrgResult]) {
    if summaries.is_empty() {
        return;
    }

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(prettytable::row![
        "Organisation",
        "#repos",
        "Fetched",
        "Failed"
    ]);

    let mut total_repos = 0;
    let mut total_fetched = 0;
    let mut total_failed = 0;

    for summary in summaries {
        table.add_row(prettytable::row![
            summary.org_name,
            r -> summary.total_repos,
            r -> summary.successful_repos,
            r -> summary.failed_repos
        ]);
        total_repos += summary.total_repos;
        total_fetched += summary.successful_repos;
        total_failed += summary.failed_repos;
    }

    table.add_empty_row();

    table.add_row(prettytable::row![
        "TOTAL",
        r -> total_repos,
        r -> total_fetched,
        r -> total_failed
    ]);

    println!("\n=== All org summary ===");
    table.printstd();
}
