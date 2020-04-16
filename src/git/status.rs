use git2::{Error, Repository, StatusOptions};

#[derive(Debug)]
pub struct GitStatus {
    pub new: Vec<String>,
    pub modified: Vec<String>,
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
    //let mut deleted = vec![];

    for entry in git_statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let status = &entry.status();
        if git2::Status::is_wt_new(status) {
            if let Some(path) = entry.path() {
                new_files.push(path.to_string());
            }
        };
        //if git2::Status::is_wt_deleted(status) {
        //if let Some(path) = entry.path {
        //deleted.push(path.to_string());
        //}
        //};
        //if git2::Status::is_wt_renamed(status) {
        //statuses_in_dir.push("renames".to_string());
        //};
        //if git2::Status::is_wt_typechange(status) {
        //statuses_in_dir.push("typechanges".to_string());
        //};
        if git2::Status::is_wt_modified(status) {
            if let Some(path) = entry.path() {
                modified.push(path.to_string());
            }
        };
    }
    let status = GitStatus {
        new: new_files,
        modified,
    };

    Ok(status)
}
