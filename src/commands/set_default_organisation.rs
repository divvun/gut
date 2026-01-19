use crate::config::Config;
use clap::Parser;

#[derive(Debug, Parser)]
/// Set default organisation name for every other command
pub struct SetOrganisationArgs {
    /// Organisation name
    #[arg(short, long)]
    pub organisation: String,
}

impl SetOrganisationArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut config = Config::from_file()?;
        config.default_org = Some(self.organisation.clone());
        config.save_config()
    }
}
