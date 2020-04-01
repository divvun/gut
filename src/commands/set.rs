use super::set_team_permission::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum SetArgs {
    #[structopt(name = "permission")]
    Persmission(SetTeamPermissionArgs),
}

impl SetArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            SetArgs::Persmission(args) => args.set_permission(),
        }
    }
}
