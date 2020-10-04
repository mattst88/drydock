use anyhow::{self, Context};

use crate::parse;

use std::fs;
use std::path::{Path, PathBuf};

const PARENT_FILE: &'static str = "parent";

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Profile {
    pub name: String,
    pub parents: Vec<ProfileKey>,
}

impl Profile {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            parents: Vec::new(),
        }
    }

    pub fn parse_parents(profile_path: &Path) -> anyhow::Result<Vec<(Option<String>, PathBuf)>> {
        let file_path = profile_path.join(PARENT_FILE);
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(file_path)?;
        parse::parse_parent_file(&contents)
    }
}

/// A type representing the unambiguous name & location of a profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileKey {
    data: String,
}

impl ProfileKey {
    pub fn new<T: Into<String>, U: Into<String>>(overlay: T, name: U) -> Self {
        Self {
            data: format!("{}:{}", overlay.into(), name.into()),
        }
    }

    pub fn overlay(&self) -> &str {
        self.data.split(":").nth(0).unwrap()
    }

    pub fn profile(&self) -> &str {
        self.data.split(":").nth(1).unwrap()
    }

    pub fn full_name(&self) -> &str {
        self.data.as_str()
    }
}
