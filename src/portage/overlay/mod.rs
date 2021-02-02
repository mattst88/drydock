mod builder;
mod traversal;

use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;

use std::fs;
use std::path::{Path, PathBuf};

use self::builder::OverlayTableBuilder;

use super::{
    profile::is_builtin_incremental_variable, profile_parser::Span, variables::TokenSet, Profile,
    ProfileKey,
};
use crate::portage::profile::{MuncherState, ValueMuncher};
use crate::portage::profile_parser::RVal;
use crate::{config::Config, parse};

use anyhow::{anyhow, Context};
use nom_locate::LocatedSpan;

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
                            Err(_e) => {
                                // TODO: Replace with logging.
                                // eprintln!(
                                //     "Malformed profile found at {:?}\n\tProblem: {}",
                                //     entry.path(),
                                //     e
                                // );
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
        let repo_name = parse::parse_layout_conf(LocatedSpan::new_extra(
            &layout_body,
            metadata_path.as_path(),
        ))?;
        let overlay = Overlay::new(repo_name.into(), value.into());
        Ok(overlay)
    }
}

pub fn build_overlay_map(config: &Config) -> anyhow::Result<OverlayTable> {
    let mut walker = ignore::WalkBuilder::new(&config.src_path);
    walker.filter_entry(|dir| dir.path().is_dir());
    walker.max_depth(Some(2));

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

    pub fn is_incremental_variable(&self, _profile_key: &ProfileKey, variable: &str) -> bool {
        is_builtin_incremental_variable(variable)
    }

    pub fn compute_variable<'a>(
        &'a self,
        profile_key: &'a ProfileKey,
        variable: &str,
    ) -> anyhow::Result<Vec<Span<'_>>> {
        if self.is_incremental_variable(profile_key, variable) {
            let incremental_values = self.compute_incremental_variable(profile_key, variable)?;
            let tokens = incremental_values.into_iter().map(|(s, _)| s).fold(
                TokenSet::default(),
                |mut base, next| {
                    base.merge(next);
                    base
                },
            );
            let vals: Vec<Span> = tokens.into_spans();

            Ok(vals)
        } else {
            Ok(self.compute_non_incremental_variable(profile_key, variable)?)
        }
    }

    pub fn visit_arborescence_postorder<'a>(
        &'a self,
        profile_key: &'a ProfileKey,
        visit: &mut impl FnMut(&'a ProfileKey) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let p = self
            .get(profile_key)
            .ok_or_else(|| construct_missing_profile_error(self, profile_key))?;
        for parent_key in p.parents.iter() {
            self.visit_arborescence_postorder(parent_key, visit)?;
        }
        visit(profile_key)?;
        Ok(())
    }

    fn compute_non_incremental_variable<'a>(
        &'a self,
        profile_key: &'a ProfileKey,
        variable: &str,
    ) -> anyhow::Result<Vec<Span<'_>>> {
        let mut muncher = ValueMuncher::new();
        let (vals, k) = match self.get_variable_with_inheritance(profile_key, variable)? {
            Some(v) => v,
            None => return Ok(Vec::new()),
        };
        match muncher.feed(vals, k) {
            MuncherState::Done(tokens) => Ok(tokens),
            MuncherState::Need((var, originating_profile)) => {
                Ok(self.get_needed_var(originating_profile, var.fragment(), &mut muncher)?)
            }
        }
    }

    pub fn compute_incremental_variable<'a>(
        &'a self,
        profile_key: &'a ProfileKey,
        variable: &str,
    ) -> anyhow::Result<Vec<(TokenSet<'_>, &ProfileKey)>> {
        let mut incremental_values = Vec::new();
        {
            let results = &mut incremental_values;
            let mut visitor = |p| {
                let var = self.compute_non_incremental_variable(p, variable)?;

                results.push((TokenSet::from_raw_spans(&var), p));
                Ok(())
            };

            self.visit_arborescence_postorder(profile_key, &mut visitor)?;
        }
        Ok(incremental_values)
    }

    fn get_needed_var<'a>(
        &'a self,
        profile_key: &'a ProfileKey,
        variable: &str,
        muncher: &mut ValueMuncher<'a>,
    ) -> anyhow::Result<Vec<Span<'_>>> {
        let (found, source) = self
            .get_variable_from_parents(profile_key, variable)?
            .unwrap_or_else(|| (RVal::placeholder(), profile_key));
        match muncher.feed(found, source) {
            MuncherState::Done(tokens) => Ok(tokens),
            MuncherState::Need((var, profile)) => {
                Ok(self.get_needed_var(profile, var.fragment(), muncher)?)
            }
        }
    }

    /// Returns the direct contents of variable from the profile specified by the ProfileKey.
    /// This function does *not* recurse into the inheritance tree and instead returns None
    /// if the variable is not defined in this Profile.
    fn get_variable_no_inheritance(
        &self,
        profile_key: &'_ ProfileKey,
        variable: &str,
    ) -> anyhow::Result<Option<&RVal<'_>>> {
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
    fn get_variable_from_parents(
        &self,
        profile_key: &ProfileKey,
        variable: &str,
    ) -> anyhow::Result<Option<(&RVal<'_>, &ProfileKey)>> {
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

    fn get_variable_with_inheritance<'a>(
        &'a self,
        profile_key: &'a ProfileKey,
        variable: &str,
    ) -> anyhow::Result<Option<(&RVal<'_>, &ProfileKey)>> {
        match self.get_variable_no_inheritance(profile_key, variable)? {
            Some(v) => Ok(Some((v, profile_key))),
            None => Ok(self.get_variable_from_parents(profile_key, variable)?),
        }
    }
}

impl Default for OverlayTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to construct an anyhow::Result with an informative error message when
/// the lookup of a ProfileKey fails. We return the most similar overlay name as a suggestion
/// where 'similarity' is measured by the normalized Damerau-Levenshtein distance.
fn construct_missing_profile_error(
    table: &OverlayTable,
    profile_key: &ProfileKey,
) -> anyhow::Error {
    match table.map.get(profile_key.overlay()) {
        // An overlay with the requested name exists, therefore the issue is that
        // a matching profile wasn't found. So we look for the profile with the most
        // similar name.
        Some(o) => {
            let nearest_profile = match o.profiles.keys().max_by_key(|s| {
                float_ord::FloatOrd(strsim::normalized_damerau_levenshtein(
                    profile_key.profile(),
                    s,
                ))
            }) {
                Some(p) => p,
                None => {
                    return anyhow!(
                        "The profile {} was requested, but the overlay {} contains no profiles.\
                         Full path of the overlay: {}",
                        profile_key.full_name(),
                        profile_key.overlay(),
                        o.path.display()
                    )
                }
            };

            anyhow!(
                "The overlay {} was found, but the profile {} does not exist. Did you mean: {}",
                profile_key.overlay(),
                profile_key.profile(),
                nearest_profile
            )
        }

        // An overlay with the requested name doesn't exist.
        // Find the most similar overlay name and suggest it as an alternative.
        None => {
            let nearest_overlay = match table.map.keys().max_by_key(|s| {
                float_ord::FloatOrd(strsim::normalized_damerau_levenshtein(
                    profile_key.overlay(),
                    s,
                ))
            }) {
                Some(p) => p,
                None => {
                    return anyhow!(
                        "No overlays were found! Ensure your config points at a valid checkout."
                    )
                }
            };

            anyhow!(
                "The overlay \"{}\" was not found. Did you mean: {}:{}",
                profile_key.overlay(),
                nearest_overlay,
                profile_key.profile()
            )
        }
    }
}
