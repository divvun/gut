use super::set_description::*;
use super::set_info::*;
use super::set_secret::*;
use super::set_team_permission::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum SetArgs {
    #[structopt(name = "description")]
    Description(DescriptionArgs),
    #[structopt(name = "info")]
    Info(InfoArgs),
    #[structopt(name = "permission")]
    Permission(SetTeamPermissionArgs),
    #[structopt(name = "secret")]
    Secret(SecretArgs),
}

impl SetArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            SetArgs::Description(args) => args.run(),
            SetArgs::Info(args) => args.run(),
            SetArgs::Permission(args) => args.set_permission(),
            SetArgs::Secret(args) => args.run(),
        }
    }
}
