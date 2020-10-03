use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::parse;

use super::Profile;

use anyhow;
use ignore::{self, DirEntry, WalkState};

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
    walker.filter_entry(|dir| dir.path().is_dir());
    walker.max_depth(Some(3));

    for overlay_path in config.get_array("overlay_paths").unwrap() {
        let p = overlay_path.into_str().unwrap();
        walker.add(p);
    }

    let mut table = OverlayTableBuilder::new();

    let walker = walker.build_parallel();

    walker.visit(&mut table);

    OverlayTable::from(table).map
}

#[derive(Debug)]
pub struct OverlayTable {
    pub map: HashMap<String, Overlay>,
}

impl OverlayTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl From<OverlayTableBuilder> for OverlayTable {
    fn from(other: OverlayTableBuilder) -> Self {
        Arc::try_unwrap(other.table).unwrap().into_inner().unwrap()
    }
}
#[derive(Debug)]
pub struct OverlayTableBuilder {
    table: Arc<Mutex<OverlayTable>>,
}

impl OverlayTableBuilder {
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(OverlayTable::new())),
        }
    }
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for OverlayTableBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(OverlayTablePiece {
            table: Arc::clone(&self.table),
            map: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub struct OverlayTablePiece {
    table: Arc<Mutex<OverlayTable>>,
    map: HashMap<String, Overlay>,
}

impl Drop for OverlayTablePiece {
    fn drop(&mut self) {
        let mut table = self.table.lock().unwrap();
        let map = std::mem::replace(&mut self.map, HashMap::new());
        for (k, v) in map {
            table.map.insert(k, v);
        }
    }
}

impl ignore::ParallelVisitor for OverlayTablePiece {
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
        if let Ok(dir) = entry {
            if let Ok(overlay) = Overlay::try_from(dir.path()) {
                self.map.insert(overlay.name.clone(), overlay);
                WalkState::Skip
            } else {
                WalkState::Continue
            }
        } else {
            WalkState::Continue
        }
    }
}
