use std::cmp::{Eq, PartialEq};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::parse;
use crate::portage::profile_parser::{full_parse, ValueMap};

use anyhow::{anyhow, Context};

const PARENT_FILE: &'static str = "parent";
const MAKE_DEFAULTS: &str = "make.defaults";

rental! {
    mod rentals {
        use super::*;

        #[rental(debug)]
        pub struct ParsedFile {
            raw: String,
            map: ValueMap<'raw>
        }
    }
}

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub parents: Vec<ProfileKey>,
    full_path: PathBuf,
    pub conf: Option<rentals::ParsedFile>,
}

impl Profile {
    pub fn new<T: Into<String>>(name: T, full_path: PathBuf) -> Self {
        Self {
            name: name.into(),
            parents: Default::default(),
            full_path,
            conf: Default::default(),
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

    pub fn parse_conf(&mut self) -> anyhow::Result<()> {
        match self.conf {
            Some(_) => Ok(()),
            None => {
                let conf_path = self.full_path.join(MAKE_DEFAULTS);
                let contents = if conf_path.is_file() {
                    fs::read_to_string(conf_path)?
                } else {
                    String::new()
                };
                match rentals::ParsedFile::try_new(contents, |x| full_parse(x)) {
                    Ok(rentref) => {
                        self.conf = Some(rentref);
                        Ok(())
                    }
                    Err(e) => panic!(e),
                }
            }
        }
    }

}

impl Hash for Profile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.parents.hash(state);
        self.full_path.hash(state);
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.parents == other.parents
            && self.full_path == other.full_path
    }
}
impl Eq for Profile {}

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
