use super::common;

use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

#[derive(Debug, Parser)]
/// Rerun the most recent workflow or send a repository_dispatch event to trigger workflows
///
/// Without "dispatch" flag this will try to re-run the most recent workflow. But This only works when the most recent workflow failed.
///
/// With "dispatch" flag, this will send a repository_dispatch event to trigger supported workflows.
/// In order to use this option. The workflow files need to use repository_dispatch event.
/// And this event will only trigger a workflow run if the workflow file is on the main or default branch.
pub struct WorkflowRunArgs {
    #[arg(long, short, alias = "organisation")]
    /// Target owner (organisation or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
    #[arg(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[arg(long, short)]
    /// Optional workflow_file_name
    pub workflow: Option<String>,
    #[arg(long, short)]
    /// Send repository_dispatch to trigger workflow rerun
    pub dispatch: bool,
}

impl WorkflowRunArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let owner = common::owner(self.owner.as_deref())?;

        let filtered_repos =
            common::query_and_filter_repositories(&owner, self.regex.as_ref(), &user_token)?;

        if filtered_repos.is_empty() {
            println!(
                "There are no repositories in {} that match the pattern {:?}",
                owner, self.regex
            );
            return Ok(());
        }

        filtered_repos.par_iter().for_each(|repo| {
            let status =
                rerun_workflow(repo, &user_token, self.workflow.as_deref(), self.dispatch);

            match status {
                Ok(s) => match s {
                    Status::SuccessByDispatch => println!(
                        "Successful to send a repository_dispatch trigger to rerun workflows for repo {}",
                        repo.name
                    ),
                    Status::Success => println!(
                        "Successful rerun the most recent workflow run for repo {}",
                        repo.name
                    ),
                    Status::NoWorkflowRunFound => {
                        println!("There is no workflow run in repo {}", repo.name)
                    }
                },
                Err(e) => println!(
                    "Failed to rerun workflow in repo {} because {:?}",
                    repo.name, e
                ),
            }
        });

        Ok(())
    }
}

fn rerun_workflow(
    repo: &RemoteRepo,
    token: &str,
    workflow: Option<&str>,
    dispatch: bool,
) -> Result<Status> {
    if dispatch {
        github::send_a_dispatch(repo, token)?;
        return Ok(Status::SuccessByDispatch);
    }

    let workflow_runs = match workflow {
        Some(wf) => github::get_workflow_runs(repo, wf, token)?,
        None => github::get_repo_workflow_runs(repo, token)?,
    };

    if workflow_runs.is_empty() {
        return Ok(Status::NoWorkflowRunFound);
    }

    let first_workflow = &workflow_runs[0];
    github::rerun_a_workflow(repo, first_workflow.id, token)?;

    Ok(Status::Success)
}

enum Status {
    NoWorkflowRunFound,
    Success,
    SuccessByDispatch,
}
