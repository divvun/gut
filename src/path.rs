use anyhow::{anyhow, Context};
use std::fs;
use std::fs::{create_dir_all, write};
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

pub fn local_path_repo(organisation: &str, name: &str, root: &str) -> PathBuf {
    let root_dir = Path::new(&root);
    root_dir.join(organisation).join(name)
}

pub fn local_path_org(organisation: &str, root: &str) -> anyhow::Result<PathBuf> {
    let root_dir = Path::new(&root);
    let local_path = root_dir.join(organisation);
    if !local_path.is_dir() {
        return Err(anyhow!(
            "There is no \"{}\" directory in root directory \"{}\"",
            organisation,
            root
        ));
    }
    Ok(local_path)
}

pub trait EnsureDirExists: Sized {
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

pub fn parrent(path: &PathBuf) -> anyhow::Result<String> {
    let parrent = path
        .parent()
        .with_context(|| format!("{:?}, there is no parent for this path", path))?
        .to_str()
        .with_context(|| format!("{:?}, directory name must be in utf-8", path))?
        .to_string();

    Ok(parrent)
}

pub fn write_content(file_path: &PathBuf, content: &str) -> anyhow::Result<()> {
    let parrent = parrent(file_path)?;
    create_dir_all(&parrent)?;
    write(file_path, content)?;
    Ok(())
}
