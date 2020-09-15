use anyhow::{self, Context};

use super::Overlay;
use crate::parse;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const PARENT_FILE: &'static str = "parent";

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

pub fn explore<'a>(
    profile: Profile<'a>,
    profile_map: &mut HashMap<Profile<'a>, Vec<Profile<'a>>>,
    overlay_map: &'a HashMap<String, Overlay>,
) -> anyhow::Result<()> {
    if let Ok(parent_file) = fs::read_to_string(profile.full_path().join(PARENT_FILE)) {
        for (overlay_name, raw_path) in parse::parse_parent_file(&parent_file) {
            let new_profile: Profile = match overlay_name {
                Some(overlay_name) => {
                    let target_overlay = overlay_map.get(&overlay_name).unwrap();
                    target_overlay.profile_from(raw_path)?
                }
                None => profile.create_relative(raw_path)?,
            };

            let parent_list = profile_map.entry(profile.clone()).or_insert(Vec::new());
            if parent_list.iter().find(|p| p == &&new_profile).is_none() {
                parent_list.push(new_profile);
            }
        }
        let frontier = profile_map.get(&profile).cloned().unwrap_or_default();
        for p in frontier {
            explore(p, profile_map, overlay_map)?
        }
    }
    Ok(())
}
