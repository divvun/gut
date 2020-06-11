use super::branch_default::*;
use super::branch_protect::*;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set default, set protected branch
pub enum BranchArgs {
    #[structopt(name = "default")]
    Default(DefaultBranchArgs),
    #[structopt(name = "protect")]
    Protect(ProtectedBranchArgs),
}

impl BranchArgs {
    pub fn run(&self) -> Result<()> {
        match self {
            BranchArgs::Default(args) => args.set_default_branch(),
            BranchArgs::Protect(args) => args.set_protected_branch(),
        }
    }
}
