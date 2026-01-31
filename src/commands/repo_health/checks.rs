use super::types::{Issue, IssueData, LFS_POINTER_MAX_BYTES, LFS_POINTER_PREFIX};
use crate::git;
use crate::path;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use unicode_normalization::UnicodeNormalization;

/// Check a single repository for all issue types
pub fn check_repo(
    repo_path: &PathBuf,
    large_file_threshold: u64,
    filename_threshold: usize,
    path_threshold: usize,
) -> Vec<Issue> {
    let repo_name = path::dir_name(repo_path).unwrap_or_default();

    // Try to open as git repo
    let git_repo = match git::open(repo_path) {
        Ok(r) => r,
        Err(e) => {
            log::debug!("Skipping {}: not a git repository ({})", repo_name, e);
            return Vec::new();
        }
    };

    let mut issues = Vec::new();

    match check_repo_for_nfc_issues(&git_repo, &repo_name) {
        Ok(nfc_issues) => issues.extend(nfc_issues),
        Err(e) => log::debug!("NFC check failed for {}: {}", repo_name, e),
    }

    match check_repo_for_case_duplicates(&git_repo, &repo_name) {
        Ok(case_issues) => issues.extend(case_issues),
        Err(e) => log::debug!("Case duplicate check failed for {}: {}", repo_name, e),
    }

    match check_repo_for_large_files_and_long_paths(
        &git_repo,
        &repo_name,
        large_file_threshold,
        filename_threshold,
        path_threshold,
    ) {
        Ok(large_path_issues) => issues.extend(large_path_issues),
        Err(e) => log::debug!(
            "Large files/long paths check failed for {}: {}",
            repo_name,
            e
        ),
    }

    issues
}

/// Build a full file path from tree walk path prefix and entry name
fn build_full_path(path_prefix: &str, name: &str) -> String {
    if path_prefix.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", path_prefix.trim_end_matches('/'), name)
    }
}

/// Get the HEAD tree from a repository, returning None for empty repos
fn get_head_tree(repo: &git2::Repository) -> Option<git2::Tree<'_>> {
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    commit.tree().ok()
}

/// Check a single repository for NFC normalization issues
///
/// This function walks the git tree and identifies filenames that are stored in NFD
/// (decomposed) form when an NFC (composed) equivalent exists in Unicode.
///
/// Note: Some character combinations (like Cyrillic base characters + combining macron U+0304)
/// have NO precomposed NFC form in Unicode. These are correctly stored in NFD form and will
/// NOT be flagged as issues. The function only reports files where an NFC equivalent exists
/// but the filename uses NFD instead.
fn check_repo_for_nfc_issues(git_repo: &git2::Repository, repo_name: &str) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    let tree = match get_head_tree(git_repo) {
        Some(t) => t,
        None => return Ok(issues), // Empty repo or no commits
    };

    // Walk the tree recursively
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            // Use name_bytes() to get raw bytes from git object database without normalization.
            // The name() method might apply NFC normalization depending on git config.
            let name_bytes = entry.name_bytes();

            // Check if name_bytes is valid UTF-8 and compare with NFC form
            if let Ok(name_str) = std::str::from_utf8(name_bytes) {
                let normalized: String = name_str.nfc().collect();

                // Only flag as issue if NFC form differs from current form.
                // This means an NFC equivalent exists but the file uses NFD.
                if name_str != normalized.as_str() {
                    issues.push(Issue {
                        repo: repo_name.to_owned(),
                        data: IssueData::Nfd {
                            file_path: build_full_path(path, name_str),
                        },
                    });
                }
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    Ok(issues)
}

/// Check a single repository for case-duplicate files
///
/// This function walks the git tree and identifies files that have the same name
/// when compared case-insensitively. On case-sensitive filesystems (Linux), these
/// files can coexist, but on case-insensitive filesystems (macOS/Windows), only
/// one can exist at a time. This causes confusion for git-lfs, which may check out
/// the wrong version.
///
/// Example: "File.txt" and "file.txt" are different on Linux but the same on macOS.
fn check_repo_for_case_duplicates(
    git_repo: &git2::Repository,
    repo_name: &str,
) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    let tree = match get_head_tree(git_repo) {
        Some(t) => t,
        None => return Ok(issues), // Empty repo or no commits
    };

    // Map lowercase path -> list of actual paths
    let mut path_map: HashMap<String, Vec<String>> = HashMap::new();

    // Walk the tree and collect all file paths
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            if let Ok(name_str) = std::str::from_utf8(entry.name_bytes()) {
                let full_path = build_full_path(path, name_str);

                // Use lowercase version as key for case-insensitive comparison
                let lowercase_path = full_path.to_lowercase();
                path_map.entry(lowercase_path).or_default().push(full_path);
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    // Find entries with more than one variant and convert to Issues
    let mut duplicates: Vec<Vec<String>> = path_map
        .into_values()
        .filter(|paths| paths.len() > 1)
        .collect();

    // Sort for consistent output
    duplicates.sort();

    for files in duplicates {
        issues.push(Issue {
            repo: repo_name.to_string(),
            data: IssueData::CaseDuplicate { files },
        });
    }

    Ok(issues)
}

/// Check a single repository for large files not tracked by LFS and long paths
///
/// This function walks the git tree and identifies:
/// 1. Large files that should be tracked by LFS
/// 2. Large files that match .gitignore patterns (should be removed from git entirely)
/// 3. Files with long paths or filenames
fn check_repo_for_large_files_and_long_paths(
    git_repo: &git2::Repository,
    repo_name: &str,
    threshold_bytes: u64,
    filename_threshold: usize,
    path_threshold: usize,
) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    let tree = match get_head_tree(git_repo) {
        Some(t) => t,
        None => return Ok(issues), // Empty repo or no commits
    };

    // Collect issues during tree walk, then sort after
    let mut large_files: Vec<(String, u64)> = Vec::new();
    let mut large_ignored_files: Vec<(String, u64)> = Vec::new();
    let mut long_paths: Vec<(String, usize, usize)> = Vec::new();

    // Walk the tree recursively
    tree.walk(git2::TreeWalkMode::PreOrder, |path, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            let name = std::str::from_utf8(entry.name_bytes()).unwrap_or("<invalid utf-8>");
            let full_path = build_full_path(path, name);

            // Check path and filename lengths
            let path_bytes_len = full_path.as_bytes().len();
            let filename_bytes_len = name.as_bytes().len();

            if filename_bytes_len > filename_threshold || path_bytes_len > path_threshold {
                long_paths.push((full_path.clone(), path_bytes_len, filename_bytes_len));
            }

            // Get the blob object to check its size
            let oid: git2::Oid = entry.id().into();
            match git_repo.find_blob(oid) {
                Ok(blob) => {
                    let size = blob.size();

                    // Check if file exceeds threshold
                    if size > threshold_bytes as usize {
                        // Check if it's an LFS pointer file
                        // LFS pointer files are small text files with specific format
                        let is_lfs = size < LFS_POINTER_MAX_BYTES
                            && blob.content().starts_with(LFS_POINTER_PREFIX);

                        if !is_lfs {
                            // Check if file should be ignored according to .gitignore
                            let should_ignore = git_repo
                                .status_should_ignore(std::path::Path::new(&full_path))
                                .unwrap_or(false);

                            if should_ignore {
                                large_ignored_files.push((full_path, size as u64));
                            } else {
                                large_files.push((full_path, size as u64));
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!(
                        "Failed to read blob for '{}' in {}: {}",
                        full_path,
                        repo_name,
                        e
                    );
                }
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    // Sort by size (largest first) and convert to Issues
    large_files.sort_by(|a, b| b.1.cmp(&a.1));
    for (file_path, size_bytes) in large_files {
        issues.push(Issue {
            repo: repo_name.to_owned(),
            data: IssueData::LargeFile {
                file_path,
                size_bytes,
            },
        });
    }

    large_ignored_files.sort_by(|a, b| b.1.cmp(&a.1));
    for (file_path, size_bytes) in large_ignored_files {
        issues.push(Issue {
            repo: repo_name.to_owned(),
            data: IssueData::LargeIgnoredFile {
                file_path,
                size_bytes,
            },
        });
    }

    // Sort long paths by path length (longest first)
    long_paths.sort_by(|a, b| b.1.cmp(&a.1));
    for (file_path, path_bytes, filename_bytes) in long_paths {
        issues.push(Issue {
            repo: repo_name.to_owned(),
            data: IssueData::LongPath {
                file_path,
                path_bytes,
                filename_bytes,
            },
        });
    }

    Ok(issues)
}
