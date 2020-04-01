use super::common;
use crate::config::Config;

pub fn show_config() -> anyhow::Result<()> {
    let user = common::user()?;
    let root = Config::root()?;

    println!(
        "Username: {}\nGithub token: {}\nRoot directory: {}",
        user.username, user.token, root
    );

    Ok(())
}
