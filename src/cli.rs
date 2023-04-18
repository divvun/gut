use crate::commands::{
    AddArgs, ApplyArgs, BranchArgs, CheckoutArgs, CiArgs, CleanArgs, CloneArgs, CommitArgs,
    CreateArgs, FetchArgs, HookArgs, InitArgs, InviteArgs, MakeArgs, MergeArgs, PullArgs, PushArgs,
    RemoveArgs, RenameArgs, SetArgs, ShowArgs, StatusArgs, TemplateArgs, TopicArgs, TransferArgs,
    WorkflowArgs,
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "gut", about = "git multirepo maintenance tool")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
pub enum Commands {
    #[command(name = "add")]
    Add(AddArgs),
    #[command(name = "apply", aliases = &["ap"])]
    Apply(ApplyArgs),
    #[command(name = "branch", aliases = &["br"])]
    Branch(BranchArgs),
    #[command(name = "checkout", aliases = &["co"])]
    Checkout(CheckoutArgs),
    #[command(name = "ci")]
    Ci(CiArgs),
    #[command(name = "clone", aliases = &["cl"])]
    Clone(CloneArgs),
    #[command(name = "clean")]
    Clean(CleanArgs),
    #[command(name = "commit")]
    Commit(CommitArgs),
    #[command(name = "create", aliases = &["cr"])]
    Create(CreateArgs),
    #[command(name = "fetch")]
    Fetch(FetchArgs),
    #[command(name = "hook")]
    Hook(HookArgs),
    #[command(name = "init")]
    Init(InitArgs),
    #[command(name = "invite")]
    Invite(InviteArgs),
    #[command(name = "make")]
    Make(MakeArgs),
    #[command(name = "merge")]
    Merge(MergeArgs),
    #[command(name = "pull")]
    Pull(PullArgs),
    #[command(name = "push")]
    Push(PushArgs),
    #[command(name = "remove")]
    Remove(RemoveArgs),
    #[command(name = "rename")]
    Rename(RenameArgs),
    #[command(name = "set")]
    Set(SetArgs),
    #[command(name = "show")]
    Show(ShowArgs),
    #[command(name = "status")]
    Status(StatusArgs),
    #[command(name = "template")]
    Template(TemplateArgs),
    #[command(name = "topic")]
    Topic(TopicArgs),
    #[command(name = "transfer")]
    Transfer(TransferArgs),
    #[command(name = "workflow")]
    Workflow(WorkflowArgs),
}
