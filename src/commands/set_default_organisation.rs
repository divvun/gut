use crate::config::Config;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Set default organisation name for every other command
pub struct SetOrganisationArgs {
    /// Organisation name
    #[structopt(short, long)]
    pub organisation: String,
}

impl SetOrganisationArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        let mut config = Config::from_file()?;
        config.default_org = Some(self.organisation.clone());
        config.save_config()
    }
}
