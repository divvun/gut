use anyhow::Result;
use git2::{Repository, Status};
use std::fs;

pub fn clean_working_dir(repo: &Repository) -> Result<()> {
    let statuses = repo.statuses(None)?;

    for entry in statuses.iter() {
        let status = entry.status();

        // Remove untracked files and directories
        if status.contains(Status::WT_NEW) {
            if let Some(path_str) = entry.path() {
                let full_path = repo
                    .workdir()
                    .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?
                    .join(path_str);

                if full_path.is_file() {
                    fs::remove_file(&full_path)?;
                } else if full_path.is_dir() {
                    fs::remove_dir_all(&full_path)?;
                }
            }
        }
    }

    Ok(())
}
