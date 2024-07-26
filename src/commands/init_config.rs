use std::path::PathBuf;
use crate::cli::Args as CommonArgs;
use crate::config::Config;
use crate::github;
use crate::user::User;
use clap::Parser;

fn validate_root(root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(root);

    if path.is_relative() {
        return Err(format!("{root} is not an absolute path. Root must be an absolute path"));
    }

    if path.exists() {
        if path.is_dir() {
            Ok(path)
        } else {
            Err(format!("{root} is a file. Root directory cannot be a file"))
        }
    } else {
        std::fs::create_dir_all(&path)
            .map_err(|source| format!("Cannot create root directory for path: {path:?} ({source})"))
            .map(|_| path)
    }
}

#[derive(Debug, Parser)]
/// Init configuration data
pub struct InitArgs {
    #[arg(
        long,
        short,
        default_value = dirs::home_dir().unwrap().into_os_string(),
        value_parser = clap::builder::ValueParser::new(validate_root),
    )]
    /// The root directory. This must be an absolute path.
    ///
    /// All repositories will be cloned under this directory
    pub root: PathBuf,
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
    pub fn save_config(&self, _common_args: &CommonArgs) -> anyhow::Result<()> {
        let user = match User::new(self.token.clone()) {
                Ok(user) => { user },
                Err(e) => match e.downcast_ref::<github::Unauthorized>() {
                    Some(_) => anyhow::bail!("Token is invalid. Check https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"),
                    _ => return Err(e)
                }
            };
        user.save_user()?;
        let config = Config::new(
            self.root.to_str().unwrap().to_string(),
            self.organisation.clone(),
            self.use_https,
        );
        config.save_config()
    }
}
