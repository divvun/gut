use super::config::Config;
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

fn config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir().map(|p| p.join("dadmin"))?;
    let config_dir = config_dir.ensure_dir_exists().ok()?;
    Some(config_dir)
}

pub fn config_path() -> Option<PathBuf> {
    let dir = config_dir()?;
    let config = dir.join("app.toml");
    Some(config)
}

pub fn user_path() -> Option<PathBuf> {
    let dir = config_dir()?;
    let config = dir.join("user.toml");
    Some(config)
}

pub fn local_path(organisation: &str, name: &str) -> Option<PathBuf> {
    let root = Config::root().ok()?;
    let root_dir = Path::new(&root);
    let local_path = root_dir.join(organisation).join(name);
    Some(local_path)
}

pub fn local_path_org(organisation: &str) -> anyhow::Result<PathBuf> {
    let root = Config::root()?;
    let root_dir = Path::new(&root);
    let local_path = root_dir.join(organisation);
    Ok(local_path)
}

trait EnsureDirExists: Sized {
    fn ensure_dir_exists(self) -> std::io::Result<Self>;
}

impl EnsureDirExists for std::path::PathBuf {
    fn ensure_dir_exists(self) -> std::io::Result<Self> {
        fs::create_dir_all(&self)?;
        Ok(self)
    }
}

pub fn remove_path(path: &PathBuf) -> std::io::Result<()> {
    if path.is_file() {
        std::fs::remove_file(path)
    } else {
        std::fs::remove_dir_all(path)
    }
}

pub fn dir_name(path: &PathBuf) -> anyhow::Result<String> {
    let dir_name = path
        .file_name()
        .with_context(|| format!("{:?}, directory name must be in utf-8", path))?
        .to_str()
        .with_context(|| format!("{:?}, directory name must be in utf-8", path))?
        .to_string();
    Ok(dir_name)
}
