use git2::{Error, Index, Repository, Diff, Oid, Tree, DiffOptions, DiffFile, DiffHunk, DiffLine, DiffDelta, DiffFormat};
use git2;
use std::str;
use crate::filter::Filter;
use crate::path;
use std::path::{Path, PathBuf};
use super::model::*;
use super::patch_file::*;
use super::common::*;
use anyhow::{anyhow, Result};
use structopt::StructOpt;
use std::fs::{read_to_string, write, create_dir_all};
use crate::git;
use super::patch_file;

#[derive(Debug, StructOpt)]
pub struct ApplyArgs {
    #[structopt(long, short, default_value = "divvun")]
    pub organisation: String,
    #[structopt(long, short)]
    pub regex: Option<Filter>,
    //#[structopt(long, short)]
    //pub template: String,
    //#[structopt(long, short)]
    //pub version: Option<usize>,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<()> {
        println!("Template apply args {:?}", self);

        let template_dir = Path::new("/Users/thanhle/dadmin/dadmin-test/lang-__UND__").to_path_buf();
        let target_dirs = vec![Path::new("/Users/thanhle/dadmin/dadmin-test/lang-en").to_path_buf()];

        let template_delta = temp_sample();

        for dir in target_dirs {
            match apply(&template_dir, &template_delta, &dir) {
                Ok(_) => println!("Applied success"),
                Err(e) => println!("Applied failed {:?}", e),
            }
        }
        Ok(())
    }
}

fn apply(template_dir: &PathBuf, template_delta: &TemplateDelta, target_dir: &PathBuf) -> Result<()> {
    let target_delta = target_delta_sample();

    let template_repo = git::open::open(template_dir)?;
    let target_repo = git::open::open(target_dir)?;

    let temp_current_sha = "440a996b2930deac0ea768c7de725aec4f08c1b4";
    let temp_last_sha = "ab4139e82667a373b7ca56f70bfa27c6fb116c85";

    let target_current_sha = "2c5236df24f20a347b7151535f380ac20e1d4c10";

    let generate_files = template_delta.generate_files(true);
    let diff = git::diff::diff_trees(&template_repo, temp_last_sha, temp_current_sha)?;

    //target_repo.apply(&diff, git2::ApplyLocation::Both, None)?;

    //print_stats(&diff);

    //let deltas = diff.deltas();
    //for delta in deltas {
        //println!("status {:?}", delta.status());
        //println!("number of files {:?}", delta.nfiles());
        //print_diff_file(&delta.old_file());
        //print_diff_file(&delta.new_file());
    //}

    //println!("====================");
    //diff.print(DiffFormat::Patch, |d, h, l| print_diff_line(d, h, l));

    let patch_files = diff_to_patch(&diff);
    println!("Yes patch_files \n {:?}", patch_files);

    Ok(())
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

    if _delta.new_file().path() != Some(Path::new("README.md")) {
        return true;
    }


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

