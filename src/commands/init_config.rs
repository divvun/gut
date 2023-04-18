use super::models::RootDirectory;
use crate::config::Config;
use crate::github;
use crate::user::User;
use clap::Parser;

#[derive(Debug, Parser)]
/// Init configuration data
pub struct InitArgs {
    #[arg(long, short, default_value = "RootDirectory::default()")]
    /// The root directory. This must be an absolute path.
    ///
    /// All repositories will be cloned under this directory
    pub root: RootDirectory,
    #[arg(short, long)]
    /// Github token. Gut needs github token to access your github data
    pub token: String,
    /// Default organisation
    #[arg(short, long)]
    pub organisation: Option<String>,
    /// Default to https instead of ssh when cloning repositories
    #[arg(short, long)]
    pub use_https: bool,
}

impl InitArgs {
    pub fn save_config(&self) -> anyhow::Result<()> {
        let user = match User::new(self.token.clone()) {
                Ok(user) => { user },
                Err(e) => match e.downcast_ref::<github::Unauthorized>() {
                    Some(_) => anyhow::bail!("Token is invalid. Check https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"),
                    _ => return Err(e)
                }
            };
        user.save_user()?;
        let config = Config::new(
            self.root.path.clone(),
            self.organisation.clone(),
            self.use_https,
        );
        config.save_config()
    }
}
