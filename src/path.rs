use anyhow::{Context, anyhow};
use std::fs;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn config_dir() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not determine config directory for this OS"))?
        .join("gut");
    config_dir
        .clone()
        .ensure_dir_exists()
        .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
    Ok(config_dir)
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    let dir = config_dir()?;
    Ok(dir.join("app.toml"))
}

pub fn user_path() -> anyhow::Result<PathBuf> {
    let dir = config_dir()?;
    Ok(dir.join("user.toml"))
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

pub fn parent(path: &PathBuf) -> anyhow::Result<String> {
    let parent = path
        .parent()
        .with_context(|| format!("{:?}, there is no parent for this path", path))?
        .to_str()
        .with_context(|| format!("{:?}, directory name must be in utf-8", path))?
        .to_string();

    Ok(parent)
}

pub fn write_content(file_path: &PathBuf, content: &str) -> anyhow::Result<()> {
    let parent = parent(file_path)?;
    create_dir_all(parent)?;
    write(file_path, content)?;
    Ok(())
}

pub fn all_files(dir: &PathBuf) -> Vec<String> {
    let len = if let Some(dir_str) = dir.to_str() {
        dir_str.len() + 1
    } else {
        return vec![];
    };

    let walk_dirs = WalkDir::new(dir);
    let mut files = vec![];
    for entry in walk_dirs
        .into_iter()
        //.filter_entry(|de| de.file_type().is_file())
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && let Some(str) = entry.into_path().to_str()
        {
            let (_a, b) = str.split_at(len);
            if !b.starts_with(".git/") {
                //println!("File: {}", b);
                files.push(b.to_string());
            }
        }
    }
    files
}
