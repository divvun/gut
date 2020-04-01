use crate::commands::{
    AddUsersArgs, BranchArgs, CloneArgs, CreateArgs, InitArgs, RemoveUsersArgs, SetArgs, ShowArgs,
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
    #[structopt(name = "clone", aliases = &["cl"])]
    CloneRepos(CloneArgs),
    #[structopt(name = "branch", aliases = &["br"])]
    Branch(BranchArgs),
    #[structopt(name = "au", aliases = &["add-users"])]
    AddUsers(AddUsersArgs),
    #[structopt(name = "ru", aliases = &["remove-users"])]
    RemoveUsers(RemoveUsersArgs),
    #[structopt(name = "set")]
    Set(SetArgs),
    #[structopt(name = "create", aliases = &["cr"])]
    Create(CreateArgs),
}
