use super::create_branch::*;
use super::create_discussion::*;
use super::create_repo::*;
use super::create_team::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
/// Create a team, discussion, repository, or branch
pub struct CreateArgs {
    #[command(subcommand)]
    command: CreateCommand,
}

impl CreateArgs {
    pub fn run(&self) -> Result<()> {
        self.command.run()
    }
}

#[derive(Debug, Parser)]
pub enum CreateCommand {
    #[command(name = "team")]
    Team(CreateTeamArgs),
    #[command(name = "discussion")]
    Discussion(CreateDiscussionArgs),
    #[command(name = "branch")]
    Branch(CreateBranchArgs),
    #[command(name = "repo", aliases = &["repository"])]
    Repo(CreateRepoArgs),
}

impl CreateCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Discussion(args) => args.create_discussion(),
            Self::Team(args) => args.create_team(),
            Self::Branch(args) => args.run(),
            Self::Repo(args) => args.run(),
        }
    }
}
