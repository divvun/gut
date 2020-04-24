use super::set_info::*;
use super::set_team_permission::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum SetArgs {
    #[structopt(name = "info")]
    Info(InfoArgs),
    #[structopt(name = "permission")]
    Permission(SetTeamPermissionArgs),
}

impl SetArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            SetArgs::Permission(args) => args.set_permission(),
            SetArgs::Info(args) => args.run(),
        }
    }
}
