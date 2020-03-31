use super::create_discussion::*;
use super::create_team::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum CreateArgs {
    #[structopt(name = "team")]
    Team(CreateTeamArgs),
    #[structopt(name = "discussion")]
    Discussion(CreateDiscussionArgs),
}

impl CreateArgs {
    pub fn do_create(&self) -> Result<()> {
        match self {
            CreateArgs::Discussion(args) => args.create_discusstion(),
            CreateArgs::Team(args) => args.create_team(),
        }
    }
}
