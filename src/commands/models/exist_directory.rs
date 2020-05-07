use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct ExistDirectory {
    pub path: PathBuf,
}

impl FromStr for ExistDirectory {
    type Err = DirError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_dir(s).map(|path| ExistDirectory { path })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum DirError {
    #[error("{path} is not a dir")]
    NotADir { path: String },
    #[error("{path} is not exist")]
    NotExist { path: String },
}

pub fn validate_dir(dir: &str) -> Result<PathBuf, DirError> {
    let path = Path::new(dir);

    if path.exists() {
        if path.is_dir() {
            Ok(path.to_path_buf())
        } else {
            Err(DirError::NotADir {
                path: dir.to_string(),
            })
        }
    } else {
        Err(DirError::NotExist {
            path: dir.to_string(),
        })
    }
}
