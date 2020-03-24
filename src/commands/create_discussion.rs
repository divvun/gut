use super::common;
use crate::github;
use crate::github::Unauthorized;

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateDiscussionArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub team_slug: String,
    #[structopt(long, short)]
    pub header: String,
    #[structopt(long, short)]
    pub body: String,
    #[structopt(long, short)]
    pub private: bool,
}

impl CreateDiscussionArgs {
    pub fn create_discusstion(&self) -> Result<()> {
        let token = common::get_user_token()?;

        match github::create_discusstion(
            &self.organisation,
            &self.team_slug,
            &self.header,
            &self.body,
            self.private,
            &token,
        ) {
            Ok(r) => println!(
                "You created a team discussion for team {} at {}",
                self.team_slug, r.html_url
            ),
            Err(e) => {
                if e.downcast_ref::<Unauthorized>().is_some() {
                    anyhow::bail!("User token invalid. Run dadmin init with a valid token");
                } else {
                    println!(
                        "Failed to create a disscusion for team {} because of {}",
                        self.team_slug, e
                    );
                }
            }
        }

        Ok(())
    }
}
