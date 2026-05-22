// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

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
}

impl OverlayTableBuilder {
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(OverlayTable::new())),
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
        })
    }
}

impl TryFrom<OverlayTableBuilder> for OverlayTable {
    type Error = anyhow::Error;
    fn try_from(other: OverlayTableBuilder) -> Result<Self, Self::Error> {
        Ok(Arc::try_unwrap(other.table).unwrap().into_inner()?)
    }
}

/// A callback struct used in each worker thread of [ignore::ParallelVisitor].
#[derive(Debug)]
pub struct OverlayTablePiece {
    table: Arc<Mutex<OverlayTable>>,
    map: HashMap<String, Overlay>,
}

impl Drop for OverlayTablePiece {
    fn drop(&mut self) {
        let mut table = self.table.lock().unwrap();
        table.map.extend(self.map.drain());
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
                        let mut failed = false;
                        for p in overlay.profiles.values_mut() {
                            match p.parse_and_ingest_conf().with_context(|| {
                                format!(
                                    "Failed while parsing {}",
                                    dir.path().join(&p.name).display()
                                )
                            }) {
                                Ok(_) => continue,
                                Err(e) => {
                                    eprintln!("Warning: skipping overlay {}: {}", overlay.name, e);
                                    failed = true;
                                    break;
                                }
                            }
                        }
                        if !failed {
                            self.map.insert(overlay.name.clone(), overlay);
                        }
                        WalkState::Skip
                    }
                    Err(e) => {
                        eprintln!("Warning: skipping overlay at {}: {}", dir.path().display(), e);
                        WalkState::Continue
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

    use crate::test_util::test_data_dir;

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
    fn test_table_builder_skips_bad_make_default() {
        let test_tree_dir = test_data_dir(&["broken-test-tree"]);

        let mut walker = ignore::WalkBuilder::new(&test_tree_dir);
        walker.filter_entry(|dir| dir.path().is_dir());

        let mut table = OverlayTableBuilder::new();

        let walker = walker.build_parallel();
        walker.visit(&mut table);

        // Overlay with bad make.defaults is skipped, not fatal.
        let table = OverlayTable::try_from(table).unwrap();
        assert_eq!(table.map.len(), 0);
    }
}
