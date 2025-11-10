use super::common;
use crate::cli::Args as CommonArgs;
use crate::config::Config;

pub fn show_config(_common_args: &CommonArgs) -> anyhow::Result<()> {
    let user = common::user()?;
    let root = Config::root()?;
    let organisation = match common::organisation(None) {
        Ok(s) => s,
        Err(_) => "(no default org)".to_string(),
    };
    let use_https = common::use_https()?;

    println!(
        "Username: {}\nGithub token: {}\nRoot directory: {}",
        user.username, user.token, root
    );
    println!("Default org: {}\nHttps? {}", organisation, use_https);

    Ok(())
}
