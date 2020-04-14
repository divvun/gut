use crate::commands::{
    AddArgs, ApplyArgs, BranchArgs, CheckoutArgs, CloneArgs, CreateArgs, InitArgs, PushArgs,
    RemoveArgs, SetArgs, ShowArgs, MergeArgs
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
    #[structopt(name = "add")]
    Add(AddArgs),
    #[structopt(name = "apply", aliases = &["ap"])]
    Apply(ApplyArgs),
    #[structopt(name = "branch", aliases = &["br"])]
    Branch(BranchArgs),
    #[structopt(name = "checkout", aliases = &["co"])]
    Checkout(CheckoutArgs),
    #[structopt(name = "clone", aliases = &["cl"])]
    Clone(CloneArgs),
    #[structopt(name = "create", aliases = &["cr"])]
    Create(CreateArgs),
    #[structopt(name = "init")]
    Init(InitArgs),
    #[structopt(name = "merge")]
    Merge(MergeArgs),
    #[structopt(name = "push")]
    Push(PushArgs),
    #[structopt(name = "remove")]
    Remove(RemoveArgs),
    #[structopt(name = "set")]
    Set(SetArgs),
    #[structopt(name = "show")]
    Show(ShowArgs),
}
