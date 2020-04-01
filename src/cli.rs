use crate::commands::{
    AddUsersArgs, CloneArgs, CreateArgs, DefaultBranchArgs, InitArgs, ProtectedBranchArgs,
    RemoveUsersArgs, SetTeamPermissionArgs, ShowArgs,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dadmin", about = "git multirepo maintenance tool")]
pub struct Args {
    #[structopt(subcommand)]
    pub command: Commands,
}

#[derive(Debug, StructOpt)]
pub enum Commands {
    #[structopt(name = "init")]
    Init(InitArgs),
    #[structopt(name = "show")]
    Show(ShowArgs),
    #[structopt(name = "cl", aliases = &["clone"])]
    CloneRepos(CloneArgs),
    #[structopt(name = "db", aliases = &["default-branch"])]
    DefaultBranch(DefaultBranchArgs),
    #[structopt(name = "pb", aliases = &["protected-branch"])]
    ProtectedBranch(ProtectedBranchArgs),
    #[structopt(name = "au", aliases = &["add-users"])]
    AddUsers(AddUsersArgs),
    #[structopt(name = "ru", aliases = &["remove-users"])]
    RemoveUsers(RemoveUsersArgs),
    #[structopt(name = "sp", aliases = &["set-permission"])]
    SetTeamPermission(SetTeamPermissionArgs),
    #[structopt(name = "cr", aliases = &["create"])]
    Create(CreateArgs),
}
