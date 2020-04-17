use git2::{Error, Repository, StatusOptions};

#[derive(Debug)]
pub struct GitStatus {
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
            && self.typechanges.is_empty()
            && self.conflicted.is_empty()
    }
}

pub fn status(repo: &Repository) -> Result<GitStatus, Error> {
    let mut opts = StatusOptions::new();
    opts.include_ignored(true)
        .include_untracked(true)
        .recurse_untracked_dirs(false)
        .exclude_submodules(false);

    let git_statuses = repo.statuses(Some(&mut opts))?;
    let mut new_files = vec![];
    let mut modified = vec![];
    let mut deleted = vec![];
    let mut renamed = vec![];
    let mut typechanges = vec![];
    let mut conflicted = vec![];

    for entry in git_statuses.iter() {
        let status = &entry.status();
        if git2::Status::is_wt_new(status) {
            if let Some(path) = entry.path() {
                new_files.push(path.to_string());
            }
        };
        if git2::Status::is_wt_deleted(status) {
            if let Some(path) = entry.path() {
                deleted.push(path.to_string());
            }
        };
        if git2::Status::is_wt_renamed(status) {
            if let Some(path) = entry.path() {
                renamed.push(path.to_string());
            }
        };
        if git2::Status::is_wt_typechange(status) {
            if let Some(path) = entry.path() {
                typechanges.push(path.to_string());
            }
        };
        if git2::Status::is_wt_modified(status) {
            if let Some(path) = entry.path() {
                modified.push(path.to_string());
            }
        };
        if git2::Status::is_conflicted(status) {
            if let Some(path) = entry.path() {
                conflicted.push(path.to_string());
            }
        };
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
