use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct Script {
    pub path: PathBuf,
}

impl FromStr for Script {
    type Err = ScriptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_script(s).map(|path| Script { path })
    }
}
#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("{path} is not a dir")]
    NotAFile { path: String },
    #[error("{path} is not exist")]
    NotExist { path: String },
    #[error("Cannot create root directory for path: {path} with error: {source}")]
    CannotCreateAbsPath {
        source: std::io::Error,
        path: String,
    },
}

/// We should handle both absolute and relative script file
/// So we need to check as follow:
/// 1. if script_path is an absolute file => ok
/// 2. if script_path is a relative file => convert to an absolute file => ok
/// 3. Otherwise fail
pub fn validate_script(script_path: &str) -> Result<PathBuf, ScriptError> {
    let path = Path::new(script_path);

    if path.exists() {
        if path.is_file() {
            if path.is_absolute() {
                Ok(path.to_path_buf())
            } else {
                match fs::canonicalize(&path) {
                    Ok(abs_path) => Ok(abs_path.to_path_buf()),
                    Err(e) => Err(ScriptError::CannotCreateAbsPath {
                        source: e,
                        path: script_path.to_string(),
                    }),
                }
            }
        } else {
            Err(ScriptError::NotAFile {
                path: script_path.to_string(),
            })
        }
    } else {
        Err(ScriptError::NotExist {
            path: script_path.to_string(),
        })
    }
}
