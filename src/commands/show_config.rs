use super::common;
use crate::config::Config;

pub fn show_config() -> anyhow::Result<()> {
    let user = common::get_user()?;
    let root = Config::get_root()?;

    println!(
        "Username: {}\nGithub token: {}\nRoot directory: {}",
        user.username, user.token, root
    );

    Ok(())
}
