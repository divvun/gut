use crate::github;
use crate::github::{CreateTeamResponse, Unauthorized};

use anyhow::{Context, Result};

use crate::user::User;
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
        let user_token = get_user_token()?;
        let response = create_team(self, &user_token)?;
        println!("Response {:?}", response);
        Ok(())
    }
}

fn get_user_token() -> Result<String> {
    User::get_token()
        .context("Cannot get user token from the config file. Run dadmin init with a valid token")
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
