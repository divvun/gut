use super::common;
use crate::config::Config;

pub fn show_config() -> anyhow::Result<()> {
    let user = common::user()?;
    let root = Config::root()?;
    let owner = match common::owner(None) {
        Ok(s) => s,
        Err(_) => "(no default owner)".to_string(),
    };
    let use_https = common::use_https()?;

    println!(
        "Username: {}\nGithub token: {}\nRoot directory: {}",
        user.username, user.token, root
    );
    println!("Default owner: {}\nHttps? {}", owner, use_https);

    Ok(())
}
