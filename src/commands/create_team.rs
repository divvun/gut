use super::common;
use crate::github;
use crate::github::{CreateTeamResponse, Unauthorized};

use anyhow::Result;

use clap::Parser;

#[derive(Debug, Parser)]
/// Create a new team for an organisation
pub struct CreateTeamArgs {
    #[arg(long, short)]
    /// Target organisation name
    ///
    /// You can set a default organisation in the init or set organisation command.
    pub organisation: Option<String>,
    #[arg(long, short)]
    /// Team name
    pub team_name: String,
    #[arg(long, short)]
    /// Team description
    pub description: Option<String>,
    #[arg(long, short)]
    /// Option to set a team is a secret team
    pub secret: bool,
    #[arg(long, short)]
    /// List of usernames to invite to the new created team
    pub members: Vec<String>,
}

impl CreateTeamArgs {
    pub fn create_team(&self) -> Result<()> {
        let user_token = common::user_token()?;

        match create_team(self, &user_token) {
            Ok(r) => println!(
            "You created a team named: {} successfully with id: {} and link : {}",
            self.team_name, r.id, r.html_url
        ),
            Err(e) => println!(
                "Failed to create team named: {} because of {}\n. \
                Please notice that you need to add users to your organisation before adding them to a team.",
                self.team_name,
                e
            )
        }

        Ok(())
    }
}

fn create_team(args: &CreateTeamArgs, token: &str) -> Result<CreateTeamResponse> {
    let empty = &"".to_string();
    let des: &str = args.description.as_ref().unwrap_or(empty);
    let members: Vec<String> = args.members.iter().map(|s| s.to_string()).collect();
    let organisation = common::organisation(args.organisation.as_deref())?;

    match github::create_team(
        &organisation,
        &args.team_name,
        des,
        members,
        args.secret,
        token,
    ) {
        Ok(response) => Ok(response),
        Err(e) => {
            if e.downcast_ref::<Unauthorized>().is_some() {
                anyhow::bail!("User token invalid. Run `gut init` with a valid token");
            }
            Err(e)
        }
    }
}
