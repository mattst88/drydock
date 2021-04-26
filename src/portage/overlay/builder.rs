use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use ignore::WalkState;

use super::{Overlay, OverlayTable};

/// A struct responsible for traversing the filesystem and building a collection of overlays.
#[derive(Debug)]
pub struct OverlayTableBuilder {
    pub(super) table: Arc<Mutex<OverlayTable>>,
    pub(super) errs: Arc<Mutex<Vec<anyhow::Error>>>,
}

impl OverlayTableBuilder {
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(OverlayTable::new())),
            errs: Arc::new(Mutex::new(Default::default())),
        }
    }
}

impl Default for OverlayTableBuilder {
    fn default() -> Self {
        Self::new()
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

/// A callback struct used in each worker thread of [ignore::ParallelVisitor].
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
        table.map.extend(self.map.drain());
        drop(table);

        let mut errs = self.errs.lock().unwrap();
        errs.extend(self.local_errs.drain(..));
    }
}

impl ignore::ParallelVisitor for OverlayTablePiece {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> WalkState {
        if let Ok(dir) = entry {
            match Overlay::try_from(dir.path()) {
                Ok(mut overlay) => match overlay
                    .parse_profiles()
                    .with_context(|| format!("Failed while parsing profiles of {}", &overlay.name))
                {
                    Ok(_) => {
                        for p in overlay.profiles.values_mut() {
                            match p.parse_and_ingest_conf().with_context(|| {
                                format!(
                                    "Failed while parsing {}",
                                    dir.path().join(&p.name).display()
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{Path, PathBuf};

    fn test_data_dir<I, P>(subdir_components: I) -> PathBuf
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "resources", "test"]
            .iter()
            .collect();
        dir.extend(subdir_components.into_iter());
        dir
    }

    #[test]
    fn test_table_builder_basic() -> anyhow::Result<()> {
        let test_tree_dir = test_data_dir(&["test-tree"]);

        let mut walker = ignore::WalkBuilder::new(&test_tree_dir);
        walker.filter_entry(|dir| dir.path().is_dir());

        let mut table = OverlayTableBuilder::new();

        let walker = walker.build_parallel();

        walker.visit(&mut table);

        let table = OverlayTable::try_from(table)?;

        // The table should have exactly 3 overlays: `spam`, `ham`, and `eggs`.
        assert_eq!(table.map.len(), 3);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Syntax error at line 6")]
    fn test_table_builder_fails_on_bad_make_default() {
        let test_tree_dir = test_data_dir(&["broken-test-tree"]);

        let mut walker = ignore::WalkBuilder::new(&test_tree_dir);
        walker.filter_entry(|dir| dir.path().is_dir());

        let mut table = OverlayTableBuilder::new();

        let walker = walker.build_parallel();
        walker.visit(&mut table);

        let _table = OverlayTable::try_from(table).unwrap();
    }
}
