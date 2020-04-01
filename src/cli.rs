use crate::commands::{
    AddArgs, BranchArgs, CloneArgs, CreateArgs, InitArgs, RemoveArgs, SetArgs, ShowArgs,
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
    #[structopt(name = "add")]
    Add(AddArgs),
    #[structopt(name = "remove")]
    Remove(RemoveArgs),
    #[structopt(name = "set")]
    Set(SetArgs),
    #[structopt(name = "create", aliases = &["cr"])]
    Create(CreateArgs),
}
