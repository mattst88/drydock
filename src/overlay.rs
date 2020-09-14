use std::fs;
use std::path::{Path, PathBuf};

use crate::parse;

use anyhow::{self, Context};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Overlay {
    pub name: String,
    pub path: PathBuf,
}

impl Overlay {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }

    pub fn try_from_path(root_path: &Path) -> Option<Self> {
        let metadata_path = root_path.join("metadata/layout.conf");
        let layout_body = fs::read_to_string(&metadata_path).ok()?;
        let repo_name = parse::parse_layout_conf(&layout_body)?;
        Some(Overlay::new(repo_name.into(), root_path.into()))
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

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Profile<'a> {
    pub overlay: &'a Overlay,
    pub rel_path: PathBuf,
}

impl<'a> Profile<'a> {
    pub fn full_path(&self) -> PathBuf {
        self.overlay.profiles_root().join(&self.rel_path)
    }

    pub fn create_relative(&self, raw_rel_path: PathBuf) -> anyhow::Result<Self> {
        let rough_path = self.full_path().join(raw_rel_path);
        let canon_path = rough_path
            .canonicalize()
            .with_context(|| format!("Failed to find path at {}", &rough_path.display()))?;
        let canon_relative_path = canon_path
            .strip_prefix(self.overlay.profiles_root())?
            .to_owned();
        Ok(Profile {
            overlay: self.overlay,
            rel_path: canon_relative_path,
        })
    }
}
