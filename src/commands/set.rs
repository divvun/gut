use super::set_default_organisation::*;
use super::set_info::*;
use super::set_secret::*;
use super::set_team_permission::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set information, secret for repositories or permission for a team
pub enum SetArgs {
    #[structopt(name = "info")]
    Info(InfoArgs),
    #[structopt(name = "organisation")]
    Organisation(SetOrganisationArgs),
    #[structopt(name = "permission")]
    Permission(SetTeamPermissionArgs),
    #[structopt(name = "secret")]
    Secret(SecretArgs),
}

impl SetArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            SetArgs::Info(args) => args.run(),
            SetArgs::Organisation(args) => args.run(),
            SetArgs::Permission(args) => args.set_permission(),
            SetArgs::Secret(args) => args.run(),
        }
    }
}
