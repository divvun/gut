use std::collections::HashSet;

/// Width of separator lines in output
pub const LINE_WIDTH: usize = 80;

/// Bytes per megabyte for size conversions
pub const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

/// Maximum size in bytes for a Git LFS pointer file.
/// LFS pointer files are small text files that reference the actual content.
pub const LFS_POINTER_MAX_BYTES: usize = 200;

/// Magic prefix that identifies a Git LFS pointer file
pub const LFS_POINTER_PREFIX: &[u8] = b"version https://git-lfs.github.com/spec/";

/// Lightweight tag for issue types - enables HashSet operations and exhaustive matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueKind {
    Nfd,
    CaseDuplicate,
    LargeFile,
    LargeIgnoredFile,
    LongPath,
}

/// Issue-specific data for each issue type
#[derive(Debug, Clone)]
pub enum IssueData {
    Nfd {
        file_path: String,
    },
    CaseDuplicate {
        files: Vec<String>,
    },
    LargeFile {
        file_path: String,
        size_bytes: u64,
    },
    LargeIgnoredFile {
        file_path: String,
        size_bytes: u64,
    },
    LongPath {
        file_path: String,
        path_bytes: usize,
        filename_bytes: usize,
    },
}

/// An issue found during repository health check
#[derive(Debug, Clone)]
pub struct Issue {
    pub repo: String,
    pub data: IssueData,
}

/// Result of checking a single repository
pub struct RepoCheckResult {
    pub repo_name: String,
    pub issues: Vec<Issue>,
}

impl Issue {
    pub fn kind(&self) -> IssueKind {
        match &self.data {
            IssueData::Nfd { .. } => IssueKind::Nfd,
            IssueData::CaseDuplicate { .. } => IssueKind::CaseDuplicate,
            IssueData::LargeFile { .. } => IssueKind::LargeFile,
            IssueData::LargeIgnoredFile { .. } => IssueKind::LargeIgnoredFile,
            IssueData::LongPath { .. } => IssueKind::LongPath,
        }
    }
}

/// Summary of issues found for a single owner
pub struct OwnerSummary {
    pub owner: String,
    pub total_repos: usize,
    pub issues: Vec<Issue>,
}

impl OwnerSummary {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn issue_kinds(&self) -> HashSet<IssueKind> {
        self.issues.iter().map(|i| i.kind()).collect()
    }

    pub fn has_issue_kind(&self, kind: IssueKind) -> bool {
        self.issues.iter().any(|i| i.kind() == kind)
    }

    pub fn count_of_kind(&self, kind: IssueKind) -> usize {
        self.issues.iter().filter(|i| i.kind() == kind).count()
    }

    pub fn affected_repos_for_kind(&self, kind: IssueKind) -> usize {
        self.issues
            .iter()
            .filter(|i| i.kind() == kind)
            .map(|i| i.repo.as_str())
            .collect::<HashSet<_>>()
            .len()
    }
}
