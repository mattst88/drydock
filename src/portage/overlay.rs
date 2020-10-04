use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::parse;

use super::{Profile, ProfileKey};

use anyhow::{self, Context};
use ignore::{self, DirEntry, WalkState};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Overlay {
    pub name: String,
    pub path: PathBuf,
    pub profiles: BTreeMap<String, Profile>,
}

impl Overlay {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            profiles: BTreeMap::new(),
        }
    }

    fn profiles_root(&self) -> PathBuf {
        self.path.join("profiles")
    }

    pub fn key_for(&self, profile_name: &str) -> Option<ProfileKey> {
        self.profiles
            .get(profile_name)
            .map(|p| ProfileKey::new(&self.name, &p.name))
    }

    fn parse_profiles(&mut self) -> anyhow::Result<()> {
        let profile_dir = self.profiles_root();

        if !profile_dir.is_dir() {
            return Ok(());
        }

        let walker = walkdir::WalkDir::new(&profile_dir).min_depth(0);

        for entry in walker
            .into_iter()
            .filter_entry(|e| e.path().is_dir())
            .filter_map(|e| e.ok())
        {
            let profile_name = String::from(
                entry
                    .path()
                    .strip_prefix(&profile_dir)
                    .with_context(|| {
                        format!(
                            "In overlay {}, exploring path {:?}",
                            &self.name,
                            entry.path()
                        )
                    })?
                    .to_string_lossy(),
            );

            let mut profile = Profile::new(&profile_name);

            'parent: for parent in Profile::parse_parents(entry.path()).unwrap_or_default() {
                match parent {
                    (Some(overlay), name) => profile
                        .parents
                        .push(ProfileKey::new(overlay, name.to_string_lossy())),
                    (None, rel_path) => {
                        let parent_path = match entry
                            .path()
                            .join(&rel_path)
                            .canonicalize()
                            .with_context(|| {
                                format!(
                                    "Relative path {:?} from {}:{} does not exist!",
                                    &rel_path, &self.name, &profile.name
                                )
                            }) {
                            Ok(p) => p,
                            Err(e) => {
                                eprintln!(
                                    "Malformed profile found at {:?}\n\tProblem: {}",
                                    entry.path(),
                                    e
                                );
                                continue 'parent;
                            }
                        };

                        let parent_name = parent_path
                            .strip_prefix(self.profiles_root())
                            .with_context(|| {
                                format!(
                                    "Tried to get relative path from: {:?}\n to {:?}",
                                    &parent_path,
                                    entry.path()
                                )
                            })?
                            .to_string_lossy();
                        profile
                            .parents
                            .push(ProfileKey::new(&self.name, parent_name))
                    }
                }
            }

            self.profiles.insert(profile_name, profile);
        }

        Ok(())
    }
}

impl TryFrom<&Path> for Overlay {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let metadata_path = value.join("metadata/layout.conf");
        let layout_body = fs::read_to_string(&metadata_path)?;
        let repo_name = parse::parse_layout_conf(&layout_body)?;
        let mut overlay = Overlay::new(repo_name.into(), value.into());
        overlay.parse_profiles()?;
        Ok(overlay)
    }
}

pub fn build_overlay_map(config: &config::Config) -> OverlayTable {
    let mut walker = ignore::WalkBuilder::new(".");
    walker.filter_entry(|dir| dir.path().is_dir());
    walker.max_depth(Some(1));

    for overlay_path in config.get_array("overlay_paths").unwrap() {
        let p = overlay_path.into_str().unwrap();
        walker.add(p);
    }

    let mut table = OverlayTableBuilder::new();

    let walker = walker.build_parallel();

    walker.visit(&mut table);

    OverlayTable::from(table)
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
            match Overlay::try_from(dir.path()) {
                Ok(overlay) => {
                    self.map.insert(overlay.name.clone(), overlay);
                    WalkState::Skip
                }
                Err(_) => WalkState::Continue,
            }
        } else {
            WalkState::Continue
        }
    }
}
