use crate::toml::{from_string, read_file, write_to_file};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn save(map: &HashMap<String, RepoData>, path: &PathBuf) -> Result<()> {
    write_to_file(path, map)
}

pub fn get(path: &PathBuf) -> Result<HashMap<String, RepoData>> {
    read_file(path)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoData {
    pub package: HashMap<String, String>,
    pub spellers: HashMap<String, Speller>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    package: Package,
    spellers: HashMap<String, Speller>,
    bundles: HashMap<String, Bundle>,
}

impl Manifest {
    #[allow(dead_code)]
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        write_to_file(path, self)
    }

    pub fn to_content(&self) -> Result<String> {
        toml::to_string(self).context("Manifest serialize error")
    }

    #[allow(dead_code)]
    pub fn get_from_file(path: &PathBuf) -> Result<Manifest> {
        read_file(path)
    }

    pub fn get_from_content(content: &str) -> Result<Manifest> {
        from_string(&content)
    }

    pub fn set_spellers(&self, spellers: &HashMap<String, Speller>) -> Manifest {
        Manifest {
            package: self.package.clone(),
            spellers: spellers.clone(),
            bundles: self.bundles.clone(),
        }
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
