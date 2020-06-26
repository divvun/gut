use super::common;
use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::{anyhow, Error, Result};
use colored::*;
use prettytable::{cell, format, row, Cell, Row, Table};
use rayon::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Add all matched repositories to a team by using team_slug
pub struct AddRepoArgs {
    #[structopt(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short, default_value = "pull", parse(try_from_str = parse_permission))]
    ///The permission to grant the team on repositories
    ///
    /// Can be one of:
    ///
    /// pull | push | admin | maintain | triage
    pub permission: String,
    #[structopt(long, short)]
    /// optional team slug
    #[structopt(long, short)]
    pub team_slug: String,
}

impl AddRepoArgs {
    pub fn run(&self) -> Result<()> {
        let user = common::user()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&organisation, self.regex.as_ref(), &user.token)?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} that matches pattern {:?}",
                organisation, self.regex
            );
            return Ok(());
        }

        let statuses: Vec<_> = filtered_repos
            .par_iter()
            .map(|r| add_repo_to_team(&r, &self.team_slug, &self.permission, &user.token))
            .collect();

        summarize(&statuses, &self.team_slug);

        //for repo in filtered_repos {
        //let result =
        //github::add_repo_to_team(&repo, &self.team_slug, &self.permission, &user.token);

        //match result {
        //Ok(_) => println!(
        //"Added repo {}/{} to team {} successfully",
        //repo.owner, repo.name, self.team_slug
        //),
        //Err(e) => println!(
        //"Failed to add repo {}/{} to team {} because {:?}",
        //repo.owner, repo.name, self.team_slug, e
        //),
        //}
        //}

        Ok(())
    }
}

fn add_repo_to_team(repo: &RemoteRepo, team: &str, permission: &str, token: &str) -> Status {
    let result = github::add_repo_to_team(repo, team, permission, token);
    Status {
        repo: repo.clone(),
        result,
    }
}

struct Status {
    repo: RemoteRepo,
    result: Result<(), Error>,
}

impl Status {
    fn to_row(&self) -> Row {
        Row::new(vec![cell!(b -> &self.repo.name), self.status()])
    }

    fn status(&self) -> Cell {
        match &self.result {
            Ok(_) => cell!(Fgr -> "Success"),
            Err(_) => cell!(Frr -> "Failed"),
        }
    }

    fn has_error(&self) -> bool {
        matches!(self.result, Err(_))
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
    table.set_titles(row!["Repo", "Status"]);
    for status in statuses {
        table.add_row(status.to_row());
    }
    table
}

fn summarize(statuses: &[Status], team_slug: &str) {
    let table = to_table(statuses);
    table.printstd();

    let errors: Vec<_> = statuses.iter().filter(|s| s.has_error()).collect();
    let successes: Vec<_> = statuses.iter().filter(|s| !s.has_error()).collect();

    if !successes.is_empty() {
        let msg = format!(
            "\nAdded {} repos to {} team successfully!",
            successes.len(),
            team_slug
        );
        println!("{}", msg.green());
    }

    if errors.is_empty() {
        println!("\nThere is no error!");
    } else {
        let msg = format!("There {} errors when executing the command:", errors.len());
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

fn parse_permission(src: &str) -> Result<String> {
    let roles = ["pull", "push", "admin", "maintain", "triage"];
    let src = src.to_lowercase();
    if roles.contains(&src.as_str()) {
        return Ok(src);
    }

    Err(anyhow!("permission must be one of {:?}", roles))
}
