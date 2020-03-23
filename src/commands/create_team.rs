use super::common;
use crate::github;
use crate::github::{CreateTeamResponse, Unauthorized};

use anyhow::Result;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateTeamArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub team_name: String,
    #[structopt(long, short)]
    pub description: Option<String>,
    #[structopt(long, short)]
    pub secret: bool,
    #[structopt(long, short)]
    pub members: Vec<String>,
}

impl CreateTeamArgs {
    pub fn create_team(&self) -> Result<()> {
        let user_token = common::get_user_token()?;

        match create_team(self, &user_token) {
            Ok(r) => println!(
            "You created a team named: {} successfully with id: {} and link : {}",
            self.team_name, r.id, r.html_url
        ),
            Err(e) => println!("Failed to create team named: {} because of {}\n. Please notice that you need to add users to your organisation before adding them to a team.", self.team_name, e)
        }

        Ok(())
    }
}

fn create_team(args: &CreateTeamArgs, token: &str) -> Result<CreateTeamResponse> {
    let empty = &"".to_string();
    let des: &str = args.description.as_ref().unwrap_or(empty);
    let members: Vec<String> = args.members.iter().map(|s| s.to_string()).collect();
    match github::create_team(
        &args.organisation,
        &args.team_name,
        des,
        members,
        args.secret,
        token,
    ) {
        Ok(response) => Ok(response),
        Err(e) => {
            if e.downcast_ref::<Unauthorized>().is_some() {
                anyhow::bail!("User token invalid. Run dadmin init with a valid token");
            }
            Err(e)
        }
    }
}
