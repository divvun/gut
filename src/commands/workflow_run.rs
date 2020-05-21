use super::common;

use crate::filter::Filter;
use crate::github;
use crate::github::RemoteRepo;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Rerun the most recent workflow
pub struct WorkflowRunArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// Optional workflow_file_name
    pub workflow: Option<String>,
}

impl WorkflowRunArgs {
    pub fn run(&self) -> Result<()> {
        let user_token = common::user_token()?;
        let filtered_repos = common::query_and_filter_repositories(
            &self.organisation,
            self.regex.as_ref(),
            &user_token,
        )?;

        if filtered_repos.is_empty() {
            println!(
                "There is no repositories in organisation {} matches pattern {:?}",
                self.organisation, self.regex
            );
            return Ok(());
        }

        for repo in filtered_repos {
            let status = rerun_workflow(&repo, &user_token, self.workflow.as_deref());

            match status {
                Ok(s) => match s {
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
        }

        Ok(())
    }
}

fn rerun_workflow(repo: &RemoteRepo, token: &str, workflow: Option<&str>) -> Result<Status> {
    let workflow_runs = match workflow {
        Some(wf) => github::get_workflow_runs(repo, wf, token)?,
        None => github::get_repo_workflow_runs(repo, token)?,
    };

    if workflow_runs.is_empty() {
        return Ok(Status::NoWorkflowRunFound);
    }

    //let first_workflow = &workflow_runs[0];

    //println!("First workflow {:?}", first_workflow);
    github::send_a_dspatch(repo, token)?;

    Ok(Status::Success)
}

enum Status {
    NoWorkflowRunFound,
    Success,
}
