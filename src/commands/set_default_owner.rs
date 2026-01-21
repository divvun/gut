use crate::config::Config;
use clap::Parser;

#[derive(Debug, Parser)]
/// Set default owner (organization or user) name for every other command
pub struct SetOwnerArgs {
    /// Owner name (GitHub organization or user account)
    #[arg(short, long, alias = "organisation")]
    pub owner: String,
}

impl SetOwnerArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut config = Config::from_file()?;
        config.default_owner = Some(self.owner.clone());
        config.save_config()
    }
}
