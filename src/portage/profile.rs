use std::path::PathBuf;

use super::Overlay;

use anyhow::{self, Context};

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
