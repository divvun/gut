use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::str;
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
                match fs::canonicalize(path) {
                    Ok(abs_path) => Ok(abs_path),
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

impl Script {
    pub fn execute_and_get_output(&self, name: &str, org: &str) -> anyhow::Result<String> {
        let script_path = self.script_path()?;
        let output = execute_script(&script_path, name, org)?;
        if output.status.success() {
            let stdout = str::from_utf8(&output.stdout)?;
            log::info!("Out put of the script: {}", stdout);
            Ok(stdout.to_string())
        } else {
            let err_message = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| format!("Cannot execute the script {}", script_path));
            Err(anyhow::anyhow!(err_message))
        }
    }

    pub fn execute_and_get_output_with_dir(
        &self,
        dir: &PathBuf,
        name: &str,
        org: &str,
    ) -> anyhow::Result<String> {
        let script_path = self.script_path()?;
        let output = execute_script_with_dir(&script_path, dir, name, org)?;
        let stdout = str::from_utf8(&output.stdout)?;
        if !stdout.is_empty() {
            let stdout = str::from_utf8(&output.stdout)?;
            log::info!("Out put of the script: {}", stdout);
            Ok(stdout.to_string())
        } else {
            let err_message = String::from_utf8(output.stderr)
                .unwrap_or_else(|_| format!("Cannot execute the script {}", script_path));
            Err(anyhow::anyhow!(err_message))
        }
    }

    pub fn script_path(&self) -> anyhow::Result<String> {
        let script_path = self
            .path
            .to_str()
            .expect("gut only supports UTF-8 paths now!");
        Ok(script_path.to_string())
    }
}

fn execute_script(script: &str, name: &str, org: &str) -> anyhow::Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", script])
            .arg(name)
            .arg(org)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg(script)
            .arg(name)
            .arg(org)
            .output()
            .expect("failed to execute process")
    };

    log::debug!("Script result {:?}", output);

    Ok(output)
}

fn execute_script_with_dir(
    script: &str,
    dir: &PathBuf,
    name: &str,
    org: &str,
) -> anyhow::Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", script])
            .arg(name)
            .arg(org)
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg(script)
            .arg(name)
            .arg(org)
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    };

    log::debug!("Script result {:?}", output);

    Ok(output)
}
