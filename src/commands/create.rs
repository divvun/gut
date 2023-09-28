use crate::cli::Args as CommonArgs;
use super::create_branch::*;
use super::create_discussion::*;
use super::create_repo::*;
use super::create_team::*;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct CreateArgs {
    #[command(subcommand)]
    command: CreateCommand,
}
/// Create team, discussion, repo to an organisation or create a branch for repositories
impl CreateArgs {
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        self.command.run(common_args)
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
    pub fn run(&self, common_args: &CommonArgs) -> Result<()> {
        match self {
            Self::Discussion(args) => args.create_discusstion(common_args),
            Self::Team(args) => args.create_team(common_args),
            Self::Branch(args) => args.run(common_args),
            Self::Repo(args) => args.run(common_args),
        }
    }
}
