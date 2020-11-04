use crate::commands::{
    AddArgs, ApplyArgs, BranchArgs, CheckoutArgs, CiArgs, CleanArgs, CloneArgs, CommitArgs,
    CreateArgs, FetchArgs, HookArgs, InitArgs, InviteArgs, MakeArgs, MergeArgs, PullArgs, PushArgs,
    RemoveArgs, RenameArgs, SetArgs, ShowArgs, StatusArgs, TemplateArgs, TopicArgs, TransferArgs, WorkflowArgs,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gut", about = "git multirepo maintenance tool")]
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
    #[structopt(name = "ci")]
    Ci(CiArgs),
    #[structopt(name = "clone", aliases = &["cl"])]
    Clone(CloneArgs),
    #[structopt(name = "clean")]
    Clean(CleanArgs),
    #[structopt(name = "commit")]
    Commit(CommitArgs),
    #[structopt(name = "create", aliases = &["cr"])]
    Create(CreateArgs),
    #[structopt(name = "fetch")]
    Fetch(FetchArgs),
    #[structopt(name = "hook")]
    Hook(HookArgs),
    #[structopt(name = "init")]
    Init(InitArgs),
    #[structopt(name = "invite")]
    Invite(InviteArgs),
    #[structopt(name = "make")]
    Make(MakeArgs),
    #[structopt(name = "merge")]
    Merge(MergeArgs),
    #[structopt(name = "pull")]
    Pull(PullArgs),
    #[structopt(name = "push")]
    Push(PushArgs),
    #[structopt(name = "remove")]
    Remove(RemoveArgs),
    #[structopt(name = "rename")]
    Rename(RenameArgs),
    #[structopt(name = "set")]
    Set(SetArgs),
    #[structopt(name = "show")]
    Show(ShowArgs),
    #[structopt(name = "status")]
    Status(StatusArgs),
    #[structopt(name = "template")]
    Template(TemplateArgs),
    #[structopt(name = "topic")]
    Topic(TopicArgs),
    #[structopt(name = "transfer")]
    Transfer(TransferArgs),
    #[structopt(name = "workflow")]
    Workflow(WorkflowArgs),
}
