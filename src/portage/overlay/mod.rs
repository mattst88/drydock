mod builder;
mod traversal;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryFrom;
use std::fs;
use std::path::{Path, PathBuf};

use self::{builder::OverlayTableBuilder, traversal::ProfileIter};

use super::{profile::is_incremental_variable, Profile, ProfileKey};
use crate::parse;
use crate::portage::profile::{MuncherState, ValueMuncher};
use crate::portage::profile_parser::RVal;

use anyhow::{anyhow, Context};

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

    pub fn compute_variable(
        &self,
        profile_key: &ProfileKey,
        variable: &str,
    ) -> anyhow::Result<String> {
        if is_incremental_variable(variable) {
            let incremental_values = self.compute_incremental_variable(profile_key, variable)?;
            let vals: Vec<String> = incremental_values
                .into_iter()
                .map(|(s, p)| {
                    dbg!(p.full_name());
                    s
                })
                .collect();
            let mut token_set = HashSet::new();

            for val in vals.iter() {
                for token in val.split_ascii_whitespace() {
                    if token.starts_with('-') {
                        token_set.remove(&token.strip_prefix('-').unwrap());
                    } else {
                        token_set.insert(token);
                    }
                }
            }

            let mut tokens: Vec<&str> = token_set.into_iter().collect();
            tokens.sort_unstable();
            tokens.dedup();

            Ok(tokens.join(" "))
        } else {
            self.compute_non_incremental_variable(profile_key, variable)
        }
    }

    fn visit_arborescence_postorder<'a: 'b, 'b: 'c, 'c, V: FnMut(&'b ProfileKey) -> anyhow::Result<()> + 'c>(
        &'a self,
        profile_key: &'b ProfileKey,
        visit: &mut V,
    ) -> anyhow::Result<()> {
        let p = self.get(profile_key).unwrap();
        for parent_key in p.parents.iter() {
            self.visit_arborescence_postorder(parent_key, visit)?;
        }
        visit(profile_key)?;
        Ok(())
    }

    fn compute_non_incremental_variable<'a, 'b, 'c>(
        &'a self,
        profile_key: &'b ProfileKey,
        variable: &'c str,
    ) -> anyhow::Result<String> {
        let mut muncher = ValueMuncher::new();
        let (vals, k) = match self.get_variable_with_inheritance(profile_key, variable)? {
            Some(v) => v,
            None => return Ok(String::new()),
        };
        match muncher.feed(vals, k) {
            MuncherState::Done(tokens) => Ok(tokens.join("")),
            MuncherState::Need((var, originating_profile)) => {
                Ok(self.get_needed_var(originating_profile, var, &mut muncher)?)
            }
        }
    }

    fn compute_incremental_variable<'a: 'b, 'b, 'c>(
        &'a self,
        profile_key: &'b ProfileKey,
        variable: &'c str,
    ) -> anyhow::Result<Vec<(String, &'b ProfileKey)>> {
        let mut incremental_values = Vec::new();
        {
            let results = &mut incremental_values;
            let _self = &self;
            let mut visitor = |p| {
                results.push((_self.compute_non_incremental_variable(p, variable)?, p));
                Ok(())
            };

            self.visit_arborescence_postorder(profile_key, &mut visitor)?;
        }
        Ok(incremental_values)
    }

    fn get_needed_var<'a, 'b: 'a>(
        &'b self,
        profile_key: &'a ProfileKey,
        variable: &'a str,
        muncher: &'b mut ValueMuncher<'a>,
    ) -> anyhow::Result<String> {
        let (found, source) = self
            .get_variable_from_parents(profile_key, variable)?
            .unwrap_or_else(|| (RVal::placeholder(), profile_key));
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
