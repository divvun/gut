mod cli;
mod commands;
mod config;
mod convert;
mod filter;
mod git;
mod github;
mod path;
mod toml;
mod user;

use anyhow::Result;
use clap::Parser;
use cli::{Args, Commands};

fn main() -> Result<()> {
    color_backtrace::install();

    pretty_env_logger::formatted_timed_builder()
        .parse_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let common_args = Args::parse();
    log::debug!("Arguments: {:?}", common_args);

    match &common_args.command {
        Commands::Add(args) => args.run(&common_args),
        Commands::Apply(args) => args.run(&common_args),
        Commands::Branch(args) => args.run(&common_args),
        Commands::Checkout(args) => args.run(&common_args),
        Commands::Ci(args) => args.run(&common_args),
        Commands::Clone(args) => args.run(&common_args),
        Commands::Clean(args) => args.run(&common_args),
        Commands::Commit(args) => args.run(&common_args),
        Commands::Create(args) => args.run(&common_args),
        Commands::Fetch(args) => args.run(&common_args),
        Commands::Hook(args) => args.run(&common_args),
        Commands::Init(args) => args.save_config(&common_args),
        Commands::Invite(args) => args.run(&common_args),
        Commands::Merge(args) => args.run(&common_args),
        Commands::Make(args) => args.run(&common_args),
        Commands::Pull(args) => args.run(&common_args),
        Commands::Push(args) => args.run(&common_args),
        Commands::Remove(args) => args.run(&common_args),
        Commands::Rename(args) => args.run(&common_args),
        Commands::Set(args) => args.run(&common_args),
        Commands::Show(args) => args.run(&common_args),
        Commands::Status(args) => args.run(&common_args),
        Commands::Template(args) => args.run(&common_args),
        Commands::Topic(args) => args.run(&common_args),
        Commands::Transfer(args) => args.run(&common_args),
        Commands::Workflow(args) => args.run(&common_args),
    }
}
