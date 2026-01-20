use super::common;
use crate::github;
use crate::github::Unauthorized;

use anyhow::Result;

use clap::Parser;

#[derive(Debug, Parser)]
/// Create a discussion for a team in an owner
pub struct CreateDiscussionArgs {
    #[arg(long, short, alias = "organisation")]
    /// Target owner (organization or user) name
    ///
    /// You can set a default owner in the init or set owner command.
    pub owner: Option<String>,
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
    pub fn create_discusstion(&self) -> Result<()> {
        let token = common::user_token()?;
        let owner = common::owner(self.owner.as_deref())?;

        match github::create_discusstion(
            &owner,
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
