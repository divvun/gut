use super::common;
use super::models::Script;
use crate::filter::Filter;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Apply a script to all local repositories that match a pattern
pub struct ApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    /// Target organisation name
    pub organisation: String,
    #[structopt(long, short)]
    /// Optional regex to filter repositories
    pub regex: Option<Filter>,
    #[structopt(long, short)]
    /// The location of a script. This must be an absolute path.
    pub script: Script,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        let root = common::root()?;
        let sub_dirs = common::read_dirs_for_org(&self.organisation, &root, self.regex.as_ref())?;

        let script_path = self
            .script
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");

        for dir in sub_dirs {
            match common::apply_script(&dir, script_path) {
                Ok(_) => println!(
                    "Applied script {} for dir {:?} successfully",
                    script_path, dir
                ),
                Err(e) => println!(
                    "Failed to apply script {} for dir {:?} because {:?}",
                    script_path, dir, e
                ),
            }
        }

        Ok(())
    }
}
