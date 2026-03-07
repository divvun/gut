use crate::system_health;
use serde::Serialize;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize)]
pub enum LfsPullStatus {
    Success,
    Failed(String),
    NotNeeded,
    LfsNotInstalled,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct LfsFileStatus {
    pub total: usize,
    pub downloaded: usize,
    pub files: Vec<LfsFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LfsFile {
    pub name: String,
    pub oid: String,
    pub downloaded: bool,
}

/// Check if a repository uses Git LFS by looking for `filter=lfs` in `.gitattributes`.
pub fn repo_uses_lfs(repo_path: &Path) -> bool {
    let gitattributes = repo_path.join(".gitattributes");
    if let Ok(contents) = std::fs::read_to_string(gitattributes) {
        contents.lines().any(|line| {
            let line = line.trim();
            !line.starts_with('#') && line.contains("filter=lfs")
        })
    } else {
        false
    }
}

/// Run `git lfs pull` in the given repository directory.
/// Returns the status of the operation.
pub fn lfs_pull(repo_path: &Path) -> LfsPullStatus {
    if !repo_uses_lfs(repo_path) {
        return LfsPullStatus::NotNeeded;
    }

    if !system_health::is_git_lfs_installed() {
        return LfsPullStatus::LfsNotInstalled;
    }

    match Command::new("git")
        .args(["lfs", "pull"])
        .current_dir(repo_path)
        .output()
    {
        Ok(output) if output.status.success() => {
            refresh_lfs_index(repo_path);
            LfsPullStatus::Success
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            LfsPullStatus::Failed(stderr)
        }
        Err(e) => LfsPullStatus::Failed(e.to_string()),
    }
}

/// Run `git lfs pull` with output visible to the user.
/// Use this when LFS downloads are expected to be large/slow.
pub fn lfs_pull_verbose(repo_path: &Path) -> LfsPullStatus {
    if !repo_uses_lfs(repo_path) {
        return LfsPullStatus::NotNeeded;
    }

    if !system_health::is_git_lfs_installed() {
        return LfsPullStatus::LfsNotInstalled;
    }

    match Command::new("git")
        .args(["lfs", "pull"])
        .current_dir(repo_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(s) if s.success() => {
            refresh_lfs_index(repo_path);
            LfsPullStatus::Success
        }
        Ok(_) => LfsPullStatus::Failed("git lfs pull failed".to_string()),
        Err(e) => LfsPullStatus::Failed(e.to_string()),
    }
}

/// Refresh git's index stat cache for all LFS-tracked files.
///
/// After `git lfs checkout` writes actual file content to disk, it does not update
/// git's index stat cache. This leaves the cache with stale data (e.g. the size of
/// the old LFS pointer file), causing `git status` to report those files as modified
/// even though their content matches the index. Running `git update-index` for each
/// LFS file re-reads the file, applies the clean filter, and writes the correct stat
/// into the cache so subsequent `git status` calls report a clean working tree.
fn refresh_lfs_index(repo_path: &Path) {
    let ls = match Command::new("git")
        .args(["lfs", "ls-files", "-n"])
        .current_dir(repo_path)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return,
    };

    let stdout = String::from_utf8_lossy(&ls.stdout);
    let files: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    if files.is_empty() {
        return;
    }

    // Pipe filenames to `git update-index --stdin` to avoid ARG_MAX limits
    // on repos with many LFS files.
    let mut child = match Command::new("git")
        .args(["update-index", "--stdin"])
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        for file in files {
            let _ = writeln!(stdin, "{file}");
        }
    }

    let _ = child.wait();
}

/// Query LFS file download status by parsing `git lfs ls-files` output.
///
/// Each line has format: `<oid_short> <*|-> <filename>`
/// where `*` = full object (downloaded), `-` = pointer only.
pub fn lfs_file_status(repo_path: &Path) -> Option<LfsFileStatus> {
    let output = Command::new("git")
        .args(["lfs", "ls-files"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "<oid> * <name>" or "<oid> - <name>"
        if let Some((oid_and_marker, name)) = line.split_once(['*', '-']) {
            let oid = oid_and_marker.trim().to_string();
            let name = name.trim().to_string();
            let downloaded = line.contains(" * ");
            files.push(LfsFile {
                name,
                oid,
                downloaded,
            });
        }
    }

    let total = files.len();
    let downloaded = files.iter().filter(|f| f.downloaded).count();

    Some(LfsFileStatus {
        total,
        downloaded,
        files,
    })
}
