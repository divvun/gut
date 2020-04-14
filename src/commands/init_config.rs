use super::models::RootDirectory;
use crate::config::Config;
use crate::github;
use crate::user::User;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct InitArgs {
    #[structopt(long, short, default_value)]
    pub root: RootDirectory,

    #[structopt(short, long)]
    pub token: String,
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
        let config = Config::new(self.root.path.clone());
        config.save_config()
    }
}
