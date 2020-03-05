use super::path::RootDirectory;
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
    #[structopt(name = "update-config")]
    Update(ConfigArgs),
    #[structopt(name = "ls", aliases = &["list-repos"])]
    ListRepos(ListRepoArgs),
}

#[derive(Debug, StructOpt)]
pub struct InitArgs {
    #[structopt(long, short, default_value)]
    pub root: RootDirectory,

    #[structopt(short, long)]
    pub token: String,
}

#[derive(Debug, StructOpt)]
pub struct ConfigArgs {
    #[structopt(long, short, default_value = "./dadmin")]
    pub root: String,
}

#[derive(Debug, StructOpt)]
pub struct ListRepoArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
}
