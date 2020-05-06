use git2::{Error, Repository, Diff, DiffOptions, DiffFile, DiffDelta, DiffLine, DiffHunk};
use std::str;
use anyhow::Result;

pub fn diff_trees<'a>(repo: &'a Repository, old: &str, new: &str) -> Result<Diff<'a>, Error> {
    let old_tree = super::tree_from_commit_sha(repo, old)?;
    let new_tree = super::tree_from_commit_sha(repo, new)?;

    let mut opts = DiffOptions::new();
    opts.old_prefix("a");
    opts.new_prefix("b");

    repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&mut opts))
}

fn print_stats(diff: &Diff) -> Result<()> {
    let stats = diff.stats()?;

    let mut format = git2::DiffStatsFormat::FULL;
    format |= git2::DiffStatsFormat::INCLUDE_SUMMARY;

    let buf = stats.to_buf(format, 80)?;
    print!("{}", str::from_utf8(&*buf).unwrap());
    Ok(())
}

fn print_diff_file(diff_file: &DiffFile) {
    println!("path {:?}", diff_file.path());
    println!("mode {:?}", diff_file.mode());
}

fn print_diff_line(
    _delta: DiffDelta,
    _hunk: Option<DiffHunk>,
    line: DiffLine,
) -> bool {

    println!("{:?} => {:?}", _delta.old_file().path(), _delta.new_file().path());

    if let Some(hs) = _hunk {
        println!("hunk {:?}", str::from_utf8(hs.header()).unwrap());
    }
    println!("{:?} -> {:?}", line.old_lineno(), line.new_lineno());
    println!("Origin {}", line.origin());

    match line.origin() {
        '+' | '-' | ' ' => print!("{}", line.origin()),
        _ => {}
    }

    print!("{}", str::from_utf8(line.content()).unwrap());
    true
}

