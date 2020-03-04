use dirs::config_dir;
use std::path::PathBuf;

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join(".dadmin.app.config"))
}

pub fn user_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join(".dadmin.user.config"))
}
