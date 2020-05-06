use std::collections::HashMap;
use crate::toml::{read_file, write_to_file};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

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
    #[serde(serialize_with = "toml::ser::tables_last")]
    pub replacements: HashMap<String, String>,
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

pub fn temp_sample() -> TemplateDelta {
    TemplateDelta {
        name: "Language Template".to_string(),
        patterns: vec!["__UND__".to_string()],
        rev_id: 2,
        required: vec![
            "src/a.txt".to_string(),
            "src/__UND__/__UND__.txt".to_string(),
            "lang-__UND__.txt".to_string(),
        ],
        optional: vec!["b.txt".to_string()],
        ignored: vec!["c.txt".to_string()],
    }
}

pub fn target_delta_sample() -> TargetDelta {
    let mut rep = HashMap::new();
    rep.insert("__UND__".to_string(), "en".to_string());
    TargetDelta {
        template: "???".to_string(),
        replacements: rep,
        rev_id: 1,
        template_sha: "ab4139e82667a373b7ca56f70bfa27c6fb116c85".to_string(),
    }
}
