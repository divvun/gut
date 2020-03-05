use dirs::{config_dir, home_dir};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join(".dadmin.app.config"))
}

pub fn user_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join(".dadmin.user.config"))
}

pub fn validate_root(root: &str) -> Result<String, RootError> {
    let path = Path::new(root);

    if path.is_relative() {
        return Err(RootError::NotAnAbsolute {
            path: root.to_string(),
        });
    }
    if path.exists() {
        if path.is_dir() {
            return Ok(root.to_string());
        } else {
            return Err(RootError::RootIsAFile {
                path: root.to_string(),
            });
        }
    } else {
        fs::create_dir_all(root)
            .map(|_| root.to_string())
            .map_err(|source| RootError::CannotCreateRoot {
                source,
                path: root.to_string(),
            })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RootError {
    #[error("{path} is a file. Root directory cannot be a file")]
    RootIsAFile { path: String },
    #[error("Cannot create root directory for path: {path} with error: {source}")]
    CannotCreateRoot {
        source: std::io::Error,
        path: String,
    },
    #[error("{path} is not an absolute path. Root must be an absolute path")]
    NotAnAbsolute { path: String },
}

#[derive(Debug)]
pub struct RootDirectory {
    pub path: String,
}

impl FromStr for RootDirectory {
    type Err = RootError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_root(s).map(|path| RootDirectory { path })
    }
}

impl Default for RootDirectory {
    fn default() -> Self {
        RootDirectory {
            path: dirs::home_dir()
                .expect("Cannot have unknown home directory")
                .join("dadmin")
                .to_str()
                .expect("Non UTF8 path is not supported right now")
                .to_string(),
        }
    }
}

impl fmt::Display for RootDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}
