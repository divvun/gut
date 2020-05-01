use std::collections::HashMap;
use regex::{Error as RegexError, Regex, RegexBuilder};
use crate::filter::Filter;
use std::path::{Path, PathBuf};
use super::model::*;
use anyhow::{anyhow, Result};
use structopt::StructOpt;
use std::fs::{read_to_string, write};

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

        let template_dir = Path::new("/Users/thanhle/dadmin/dadmin-test/lang-UND").to_path_buf();
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

    //println!("temp dir: {:?}", template_dir);
    //println!("temp delta: {:?}", template_delta);
    //println!("target dir: {:?}", target_dir);
    //println!("target delta: {:?}", target_delta);

    //for file in &template_delta.required {
        //let file_path = template_dir.join(file);
        //let content = read_to_string(&file_path)?;
        //println!("Content of {:?}", file_path);
        //println!("{}", content);
    //}

    let rx = template_delta.required.iter().map(AsRef::as_ref).collect();
    let targetd_files = generate_file_paths(&target_delta.replacements, rx);
    Ok(())
}

fn generate_file_paths(replacements: &HashMap<String, String>, files: Vec<&str>) -> Result<Vec<String>> {
    for file in files {
        for (pattern, replace) in replacements {
            let re = to_regex(&pattern)?;
            let result = re.replace_all(file, &replace[..]);
            println!("Replace result {:?}", result);
        }
    }

    Ok(vec![])
}

fn to_regex(s: &str) -> Result<Regex, RegexError> {
    RegexBuilder::new(s)
        .case_insensitive(true)
        .build()
}
