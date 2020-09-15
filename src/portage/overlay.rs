use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parse;

use super::Profile;

use anyhow;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Overlay {
    pub name: String,
    pub path: PathBuf,
}

impl Overlay {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }

    pub fn profiles_root(&self) -> PathBuf {
        self.path.join("profiles")
    }

    pub fn profile_from(&self, rel_path: PathBuf) -> anyhow::Result<Profile<'_>> {
        let rough_path = self.profiles_root().join(rel_path);
        let canon_path = rough_path.canonicalize()?;
        let canon_relative_path = canon_path.strip_prefix(self.profiles_root())?.to_owned();
        Ok(Profile {
            overlay: &self,
            rel_path: canon_relative_path,
        })
    }
}

impl TryFrom<&Path> for Overlay {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let metadata_path = value.join("metadata/layout.conf");
        let layout_body = fs::read_to_string(&metadata_path)?;
        let repo_name = parse::parse_layout_conf(&layout_body)?;
        Ok(Overlay::new(repo_name.into(), value.into()))
    }
}
