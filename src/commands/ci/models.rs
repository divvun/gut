use crate::toml::{from_string, read_file, write_to_file};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn save(map: &BTreeMap<String, RepoData>, path: &PathBuf) -> Result<()> {
    write_to_file(path, map)
}

pub fn get(path: &PathBuf) -> Result<BTreeMap<String, RepoData>> {
    read_file(path)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoData {
    pub package: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    package: Package,
    spellers: BTreeMap<String, Speller>,
    bundles: BTreeMap<String, Bundle>,
}

impl Manifest {
    #[allow(dead_code)]
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        write_to_file(path, self)
    }

    #[allow(dead_code)]
    pub fn to_content(&self) -> Result<String> {
        toml::to_string(self).context("Manifest serialize error")
    }

    #[allow(dead_code)]
    pub fn get_from_file(path: &PathBuf) -> Result<Manifest> {
        read_file(path)
    }

    #[allow(dead_code)]
    pub fn get_from_content(content: &str) -> Result<Manifest> {
        from_string(&content)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Speller {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_win: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub human_name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bundle {
    pub package: String,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub repo: String,
}
