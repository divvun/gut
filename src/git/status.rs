use git2::{Error, Repository, Status, StatusOptions};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    pub added: Vec<String>,
    pub new: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
    pub renamed: Vec<String>,
    pub typechanges: Vec<String>,
    pub conflicted: Vec<String>,
    pub is_ahead: usize,
    pub is_behind: usize,
}

impl GitStatus {
    pub fn is_empty(&self) -> bool {
        self.new.is_empty()
            && self.modified.is_empty()
            && self.deleted.is_empty()
            && self.renamed.is_empty()
            && self.conflicted.is_empty()
            && self.added.is_empty()
    }

    pub fn can_commit(&self) -> bool {
        self.conflicted.is_empty()
    }

    pub fn addable_list(&self) -> Vec<String> {
        let list = vec![self.new.clone(), self.modified.clone()];
        list.into_iter().flatten().collect()
    }

    pub fn should_commit(&self) -> bool {
        self.can_commit() && !self.is_empty()
    }

    pub fn is_dirty(&self) -> bool {
        !self.added.is_empty()
            || !self.modified.is_empty()
            || !self.deleted.is_empty()
            || !self.renamed.is_empty()
            || !self.conflicted.is_empty()
    }

    pub fn is_not_dirty(&self) -> bool {
        self.new.is_empty()
            && self.modified.is_empty()
            && self.deleted.is_empty()
            && self.renamed.is_empty()
            && self.conflicted.is_empty()
    }

    pub fn ahead_behind(&self) -> String {
        if self.is_ahead > 0 {
            format!("{}", self.is_ahead)
        } else if self.is_behind > 0 {
            format!("-{}", self.is_behind)
        } else {
            format!("{}", 0)
        }
    }

    pub fn should_push(&self) -> bool {
        self.is_ahead > 0
    }
}

pub fn status(repo: &Repository, recurse_untracked_dirs: bool) -> Result<GitStatus, Error> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(false)
        .include_untracked(true)
        .recurse_untracked_dirs(recurse_untracked_dirs)
        .exclude_submodules(false);

    let git_statuses = repo.statuses(Some(&mut opts))?;

    let mut added = vec![];
    let mut new_files = vec![];
    let mut modified = vec![];
    let mut deleted = vec![];
    let mut renamed = vec![];
    let mut typechanges = vec![];
    let mut conflicted = vec![];

    for entry in git_statuses.iter() {
        let status = &entry.status();
        //if let Some(path) = entry.path() {
        //log::debug!("entry {:?} {}", entry.status(), path);
        //}

        if Status::is_wt_new(status) {
            if let Some(path) = entry.path() {
                new_files.push(path.to_string());
            }
        } else if Status::is_wt_deleted(status) {
            if let Some(path) = entry.path() {
                deleted.push(path.to_string());
            }
        } else if Status::is_wt_renamed(status) {
            if let Some(path) = entry.path() {
                renamed.push(path.to_string());
            }
        } else if Status::is_wt_typechange(status) {
            if let Some(path) = entry.path() {
                typechanges.push(path.to_string());
            }
        } else if Status::is_wt_modified(status) {
            if let Some(path) = entry.path() {
                modified.push(path.to_string());
            }
        } else if Status::is_conflicted(status) {
            if let Some(path) = entry.path() {
                conflicted.push(path.to_string());
            }
        } else if (Status::is_index_new(status)
            || Status::is_index_modified(status)
            || Status::is_index_deleted(status)
            || Status::is_index_renamed(status)
            || Status::is_index_typechange(status))
            && let Some(path) = entry.path()
        {
            added.push(path.to_string());
        }
    }

    //      Adapted from @Kurt-Bonatz in https://github.com/rust-lang/git2-rs/issues/332#issuecomment-408453956
    let mut is_ahead = 0;
    let mut is_behind = 0;
    if repo.revparse_single("HEAD").is_ok() {
        let head_ref = repo.revparse_single("HEAD").expect("HEAD not found").id();
        let (ahead, behind) = repo
            .revparse_ext("@{u}")
            .ok()
            .and_then(|(upstream, _)| repo.graph_ahead_behind(head_ref, upstream.id()).ok())
            .unwrap_or((0, 0));

        if ahead > 0 {
            is_ahead = ahead;
        }

        if behind > 0 {
            is_behind = behind;
        }
    }

    let status = GitStatus {
        added,
        new: new_files,
        modified,
        deleted,
        renamed,
        typechanges,
        conflicted,
        is_ahead,
        is_behind,
    };

    Ok(status)
}
