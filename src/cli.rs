use super::path::RootDirectory;
use regex::{Error as RegexError, Regex, RegexBuilder};
use std::{fmt, str::FromStr};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dadmin", about = "git multirepo maintenance tool")]
pub struct Args {
    #[structopt(flatten)]
    pub global_args: GlobalArgs,
    #[structopt(subcommand)]
    pub command: Commands,
}

#[derive(Debug, StructOpt)]
pub enum Commands {
    #[structopt(name = "init")]
    Init(InitArgs),
    #[structopt(name = "update-config")]
    Update(ConfigArgs),
    #[structopt(name = "lr", aliases = &["list-repos"])]
    ListRepos,
    #[structopt(name = "cl", aliases = &["clone-repos"])]
    CloneRepos,
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
pub struct GlobalArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub repositories: Option<Filter>,
}

#[derive(Debug)]
pub struct Filter {
    regex: Regex,
}

impl FromStr for Filter {
    type Err = RegexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RegexBuilder::new(s)
            .case_insensitive(true)
            .build()
            .map(|regex| Filter { regex })
    }
}

impl Filter {
    pub fn is_match(&self, pattern: &str) -> bool {
        self.regex.is_match(pattern)
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.regex)
    }
}
