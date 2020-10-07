use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{Profile, ProfileKey};
use crate::parse;
use crate::portage::profile::{MuncherState, ValueMuncher};
use crate::portage::profile_parser::RVal;

use anyhow::{anyhow, Context};
use ignore::{self, DirEntry, WalkState};

#[derive(Debug, Hash, Eq, PartialEq)]
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

    #[allow(dead_code)]
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

        let walker = walkdir::WalkDir::new(&profile_dir).min_depth(1);

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

            let mut profile = Profile::new(&profile_name, entry.path().into());

            for parent in Profile::parse_parents(entry.path()).unwrap_or_default() {
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
                                continue;
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
        let overlay = Overlay::new(repo_name.into(), value.into());
        Ok(overlay)
    }
}

pub fn build_overlay_map(config: &config::Config) -> anyhow::Result<OverlayTable> {
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

    OverlayTable::try_from(table)
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

    pub fn get(&self, key: &ProfileKey) -> Option<&Profile> {
        self.map
            .get(key.overlay())
            .map(|o| o.profiles.get(key.profile()))
            .flatten()
    }

    pub fn compute_flattened_variable(
        &self,
        key: &ProfileKey,
        variable: &str,
    ) -> anyhow::Result<String> {
        let mut muncher = ValueMuncher::new();
        let (vals, k) = self.get_variable_with_inheritance(key, variable)?.unwrap();
        match muncher.feed(vals, k) {
            MuncherState::Done(tokens) => Ok(tokens.join("")),
            MuncherState::Need((var, profile)) => {
                Ok(self.get_needed_var(profile, var, &mut muncher)?)
            }
        }
    }

    fn get_needed_var<'a, 'b: 'a>(
        &'b self,
        profile_key: &'a ProfileKey,
        variable: &'a str,
        muncher: &'b mut ValueMuncher<'a>,
    ) -> anyhow::Result<String> {
        let (found, source) = self
            .get_variable_from_parents(profile_key, variable)?
            .ok_or_else(|| anyhow!("Needed a value for ${{{}}}", variable))?;
        match muncher.feed(found, source) {
            MuncherState::Done(tokens) => Ok(tokens.join("")),
            MuncherState::Need((var, profile)) => Ok(self.get_needed_var(profile, var, muncher)?),
        }
    }

    /// Returns the direct contents of variable from the profile specified by the ProfileKey.
    /// This function does *not* recurse into the inheritance tree and instead returns None
    /// if the variable is not defined in this Profile.
    fn get_variable_no_inheritance<'a: 'data, 'data, 'c>(
        &'a self,
        profile_key: &'c ProfileKey,
        variable: &'data str,
    ) -> anyhow::Result<Option<&'a RVal<'data>>> {
        let profile = self.get(profile_key).ok_or_else(|| {
            anyhow!(
                "Unable to find a profile for key \"{}\"!",
                profile_key.full_name()
            )
        })?;
        Ok(profile.get(variable))
    }

    /// Given a profile, get the direct contents of a variable from the highest priority parent of
    /// that profile, *not including* the specified profile itself.
    /// The inheritance heirarchy for profiles evaluated left-to-right so we search the rightmost
    /// parent first as that is the highest priority profile.
    fn get_variable_from_parents<'a: 'data, 'data>(
        &'a self,
        profile_key: &'data ProfileKey,
        variable: &'data str,
    ) -> anyhow::Result<Option<(&'data RVal<'data>, &'data ProfileKey)>> {
        let profile = self.get(profile_key).ok_or_else(|| {
            anyhow!(
                "Unable to find a profile for key \"{}\"!",
                profile_key.full_name()
            )
        })?;

        let value = profile
            .parents
            .iter()
            .rev() // Start with the rightmost parent.
            .find_map(|parent_key| {
                self.get_variable_with_inheritance(parent_key, variable)
                    .ok()
                    .flatten()
            });

        Ok(value)
    }

    fn get_variable_with_inheritance<'a: 'data, 'data>(
        &'a self,
        profile_key: &'data ProfileKey,
        variable: &'data str,
    ) -> anyhow::Result<Option<(&'a RVal<'data>, &'data ProfileKey)>> {
        match self.get_variable_no_inheritance(profile_key, variable)? {
            Some(v) => Ok(Some((v, profile_key))),
            None => Ok(self.get_variable_from_parents(profile_key, variable)?),
        }
    }
}

impl TryFrom<OverlayTableBuilder> for OverlayTable {
    type Error = anyhow::Error;
    fn try_from(other: OverlayTableBuilder) -> Result<Self, Self::Error> {
        let errs = Arc::try_unwrap(other.errs).unwrap().into_inner().unwrap();
        if !errs.is_empty() {
            return Err(errs.into_iter().next().unwrap());
        }
        Ok(Arc::try_unwrap(other.table).unwrap().into_inner()?)
    }
}

#[derive(Debug)]
pub struct OverlayTableBuilder {
    table: Arc<Mutex<OverlayTable>>,
    errs: Arc<Mutex<Vec<anyhow::Error>>>,
}

impl OverlayTableBuilder {
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(OverlayTable::new())),
            errs: Arc::new(Mutex::new(Default::default())),
        }
    }
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for OverlayTableBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(OverlayTablePiece {
            table: Arc::clone(&self.table),
            map: HashMap::new(),
            errs: Arc::clone(&self.errs),
            local_errs: Default::default(),
        })
    }
}

#[derive(Debug)]
pub struct OverlayTablePiece {
    table: Arc<Mutex<OverlayTable>>,
    errs: Arc<Mutex<Vec<anyhow::Error>>>,
    map: HashMap<String, Overlay>,
    local_errs: Vec<anyhow::Error>,
}

impl Drop for OverlayTablePiece {
    fn drop(&mut self) {
        let mut table = self.table.lock().unwrap();
        let map = std::mem::replace(&mut self.map, HashMap::new());
        for (k, v) in map {
            table.map.insert(k, v);
        }

        let mut errs = self.errs.lock().unwrap();
        let local_errs = std::mem::take(&mut self.local_errs);
        for e in local_errs {
            errs.push(e);
        }
    }
}

impl ignore::ParallelVisitor for OverlayTablePiece {
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
        if let Ok(dir) = entry {
            match Overlay::try_from(dir.path()) {
                Ok(mut overlay) => match overlay
                    .parse_profiles()
                    .with_context(|| format!("Failed while parsing profiles of {}", &overlay.name))
                {
                    Ok(_) => {
                        for p in overlay.profiles.values_mut() {
                            match p.parse_conf().with_context(|| {
                                format!(
                                    "Failed while parsing {:?}:{} conf!",
                                    dir.path().components().last().unwrap(),
                                    p.name
                                )
                            }) {
                                Ok(_) => continue,
                                Err(e) => {
                                    self.local_errs.push(e);
                                    return WalkState::Quit;
                                }
                            }
                        }
                        self.map.insert(overlay.name.clone(), overlay);
                        WalkState::Skip
                    }
                    Err(e) => {
                        self.local_errs.push(e);
                        WalkState::Quit
                    }
                },
                Err(_) => WalkState::Continue,
            }
        } else {
            WalkState::Continue
        }
    }
}
