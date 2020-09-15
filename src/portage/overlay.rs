use std::collections::HashMap;
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

pub fn build_overlay_map(config: &config::Config) -> HashMap<String, Overlay> {
    let mut walker = ignore::WalkBuilder::new(".");
    walker.max_depth(Some(1));

    for overlay_path in config.get_array("overlay_paths").unwrap() {
        let p = dbg!(overlay_path.into_str().unwrap());
        walker.add(p);
    }

    let mut map = HashMap::new();

    for candidate_path in walker.build() {
        let candidate_path = candidate_path.unwrap();
        if let Ok(x) = Overlay::try_from(candidate_path.path()) {
            map.insert(x.name.clone(), x);
        }
    }

    map
}
