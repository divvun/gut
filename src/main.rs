mod cli;
mod commands;
mod config;
mod convert;
mod filter;
mod git;
mod github;
mod health;
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

    let args = Args::parse();
    log::debug!("Arguments: {:?}", args);

    match args.command {
        Commands::Add(cmd) => cmd.run(),
        Commands::Apply(cmd) => cmd.run(),
        Commands::Branch(cmd) => cmd.run(),
        Commands::Checkout(cmd) => cmd.run(),
        Commands::Ci(cmd) => cmd.run(),
        Commands::Clone(cmd) => cmd.run(),
        Commands::Clean(cmd) => cmd.run(),
        Commands::Commit(cmd) => cmd.run(),
        Commands::Create(cmd) => cmd.run(),
        Commands::Fetch(cmd) => cmd.run(),
        Commands::Health(cmd) => cmd.run(),
        Commands::Hook(cmd) => cmd.run(),
        Commands::Init(cmd) => cmd.save_config(),
        Commands::Invite(cmd) => cmd.run(),
        Commands::Merge(cmd) => cmd.run(),
        Commands::Make(cmd) => cmd.run(),
        Commands::Pull(cmd) => cmd.run(args.format),
        Commands::Push(cmd) => cmd.run(),
        Commands::Remove(cmd) => cmd.run(),
        Commands::Rename(cmd) => cmd.run(),
        Commands::Set(cmd) => cmd.run(),
        Commands::Show(cmd) => cmd.run(),
        Commands::Status(cmd) => cmd.run(args.format),
        Commands::Template(cmd) => cmd.run(),
        Commands::Topic(cmd) => cmd.run(),
        Commands::Transfer(cmd) => cmd.run(),
        Commands::Workflow(cmd) => cmd.run(),
    }
}
