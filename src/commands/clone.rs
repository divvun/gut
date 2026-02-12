use super::common;

use crate::github::RemoteRepo;
use anyhow::{Error, Result, anyhow};

use crate::convert::try_from_one;
use crate::filter::Filter;
use crate::git::Clonable;
use crate::git::lfs;
use crate::git::models::GitRepo;
use crate::system_health;
use crate::user::User;
use clap::Parser;
use colored::*;
use prettytable::{Cell, Row, Table, cell, format, row};
use std::process::{Command, Stdio};

#[derive(Debug, Parser)]
/// Clone all repositories that matches a pattern
pub struct CloneArgs {
    #[arg(long, short, alias = "organisation")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Option to use https instead of ssh when clone repositories
    pub use_https: bool,
}

impl CloneArgs {
    pub fn run(&self) -> Result<()> {
        let warnings = system_health::check_git_config();

        let user = common::user()?;
        let owner = common::owner(self.owner.as_deref())?;
        let use_https = match self.use_https {
            true => true,
            false => common::use_https()?,
        };

        let filtered_repos =
            common::query_and_filter_repositories(&owner, self.regex.as_ref(), &user.token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in {} that match the pattern {:?}",
                &owner, self.regex
            );
            return Ok(());
        }

        // Phase 1: Clone all repos with libgit2 (parallel, with progress bar).
        // For LFS repos this only fetches git objects (pointer files are tiny).
        let mut statuses = common::process_with_progress(
            "Cloning",
            &filtered_repos,
            |r| clone(r, &user, use_https),
            |status| status.repo.name.clone(),
        );

        // Phase 2: Re-clone LFS repos with git CLI (sequential) so that
        // LFS objects are fetched automatically and progress is visible.
        for status in &mut statuses {
            let Ok(git_repo) = &status.result else {
                continue;
            };
            if !lfs::repo_uses_lfs(&git_repo.local_path) {
                continue;
            }

            let remote_url = git_repo.remote_url.clone();
            let local_path = git_repo.local_path.clone();

            println!("\nCloning {} with LFS...", status.repo.name);

            if let Err(e) = std::fs::remove_dir_all(&local_path) {
                status.result = Err(anyhow!("Failed to remove directory for re-clone: {}", e));
                continue;
            }

            match Command::new("git")
                .args([
                    "clone",
                    "--progress",
                    &remote_url,
                    &local_path.to_string_lossy(),
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
            {
                Ok(s) if s.success() => {}
                Ok(_) => {
                    status.result = Err(anyhow!("git clone failed for {}", status.repo.name));
                }
                Err(e) => {
                    status.result = Err(anyhow!("Failed to run git clone: {}", e));
                }
            }
        }

        summarize(&statuses);

        system_health::print_warnings(&warnings);
        Ok(())
    }
}

fn clone(repo: &RemoteRepo, user: &User, use_https: bool) -> Status {
    let cl = || -> Result<GitRepo> {
        let git_repo = try_from_one(repo.clone(), user, use_https)?;
        if git_repo.local_path.exists() {
            return Err(anyhow!(
                "Repository {} already exists at {:?}",
                repo.name,
                git_repo.local_path
            ));
        }
        let result = git_repo.gclone()?;
        Ok(result)
    };
    Status {
        repo: repo.clone(),
        result: cl(),
    }
}

struct Status {
    repo: RemoteRepo,
    result: Result<GitRepo, Error>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![
            cell!(b -> &self.repo.name),
            self.status(),
            self.lfs_cell(),
        ])
    }

    fn status(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Success"),
            Err(_) => cell!(Frr -> "Failed"),
        }
    }

    fn lfs_cell(&self) -> Cell {
        match &self.result {
            Ok(git_repo) if lfs::repo_uses_lfs(&git_repo.local_path) => cell!(Fgr -> "Yes"),
            _ => cell!(r -> "-"),
        }
    }

    fn has_error(&self) -> bool {
        self.result.is_err()
    }

    fn to_error_row(&self) -> Row {
        let e = if let Err(e) = &self.result {
            e
        } else {
            panic!("This should have an error here");
        };

        let msg = format!("{:?}", e);
        let lines = common::sub_strings(msg.as_str(), 80);
        let lines = lines.join("\n");
        row!(cell!(b -> &self.repo.name), cell!(Fr -> lines.as_str()))
    }
}

fn to_table(statuses: &[Status]) -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
    table.set_titles(row!["Repo", "Status", "LFS"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn summarize(statuses: &[Status]) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!("\nCloned {} repos successfully!", successes.len());
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere were no errors!");
    } else {
        let msg = format!("There were {} errors when cloning:", errors.len());
        println!("\n{}\n", msg.red());

        let mut error_table = Table::new();
        error_table.set_format(*format::consts::FORMAT_BORDERS_ONLY);
        error_table.set_titles(row!["Repo", "Error"]);
        for error in errors {
            error_table.add_row(error.to_error_row());
        }
        error_table.printstd();
    }
}
