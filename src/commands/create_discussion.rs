use crate::cli::Args as CommonArgs;
use super::common;
use crate::github;
use crate::github::Unauthorized;

use anyhow::Result;

use clap::Parser;

#[derive(Debug, Parser)]
/// Create a discussion for a team in an organisation
pub struct CreateDiscussionArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Team slug
    pub team_slug: String,
    #[arg(long, short)]
    /// Subject of the discussion
    pub subject: String,
    #[arg(long, short)]
    /// Body of the discussion
    pub body: String,
    #[arg(long, short)]
    /// Option to set the discussion is private
    pub private: bool,
}

impl CreateDiscussionArgs {
    pub fn create_discusstion(&self, _common_args: &CommonArgs) -> Result<()> {
        let token = common::user_token()?;
        let organisation = common::organisation(self.organisation.as_deref())?;

        match github::create_discusstion(
            &organisation,
            &self.team_slug,
            &self.subject,
            &self.body,
            self.private,
            &token,
        ) {
            Ok(r) => println!(
                "You created a team discussion for team `{}` at {}",
                self.team_slug, r.html_url
            ),
            Err(e) => {
                if e.downcast_ref::<Unauthorized>().is_some() {
                    anyhow::bail!("User token invalid. Run `gut init` with a valid token");
                } else {
                    println!(
                        "Failed to create a discussion for team `{}` because of {}",
                        self.team_slug, e
                    );
                }
            }
        }

        Ok(())
    }
}
