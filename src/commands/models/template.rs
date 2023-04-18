use crate::toml::{read_file, write_to_file};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateDelta {
    pub name: String,
    pub patterns: Vec<String>,
    pub rev_id: usize,
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub ignored: Vec<String>,
}

impl TemplateDelta {
    pub fn generate_files(&self, include_optional: bool) -> Vec<String> {
        let mut files = vec![self.required.clone()];
        if include_optional {
            files.push(self.optional.clone());
        }
        files.concat()
    }

    #[allow(dead_code)]
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        write_to_file(path, self)
    }

    pub fn get(path: &PathBuf) -> Result<TemplateDelta> {
        read_file(path)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetDelta {
    pub template: String,
    pub rev_id: usize,
    pub template_sha: String,
    pub replacements: BTreeMap<String, String>,
}

impl TargetDelta {
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        write_to_file(path, self)
    }

    pub fn get(path: &PathBuf) -> Result<TargetDelta> {
        read_file(path)
    }

    pub fn update(&self, rev_id: usize, template_sha: &str) -> TargetDelta {
        TargetDelta {
            template: self.template.clone(),
            rev_id,
            template_sha: template_sha.to_string(),
            replacements: self.replacements.clone(),
        }
    }
}
