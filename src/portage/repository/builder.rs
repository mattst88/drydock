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

use super::{Repository, RepositoryTable};

/// A struct responsible for traversing the filesystem and building a collection of repositories.
#[derive(Debug)]
pub struct RepositoryTableBuilder {
    pub(super) table: Arc<Mutex<RepositoryTable>>,
}

impl RepositoryTableBuilder {
    pub fn new() -> Self {
        Self {
            table: Arc::new(Mutex::new(RepositoryTable::new())),
        }
    }
}

impl Default for RepositoryTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<'s> ignore::ParallelVisitorBuilder<'s> for RepositoryTableBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(RepositoryTablePiece {
            table: Arc::clone(&self.table),
            map: HashMap::new(),
        })
    }
}

impl TryFrom<RepositoryTableBuilder> for RepositoryTable {
    type Error = anyhow::Error;
    fn try_from(other: RepositoryTableBuilder) -> Result<Self, Self::Error> {
        Ok(Arc::try_unwrap(other.table).unwrap().into_inner()?)
    }
}

/// A callback struct used in each worker thread of [ignore::ParallelVisitor].
#[derive(Debug)]
pub struct RepositoryTablePiece {
    table: Arc<Mutex<RepositoryTable>>,
    map: HashMap<String, Repository>,
}

impl Drop for RepositoryTablePiece {
    fn drop(&mut self) {
        let mut table = self.table.lock().unwrap();
        table.map.extend(self.map.drain());
    }
}

impl ignore::ParallelVisitor for RepositoryTablePiece {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> WalkState {
        if let Ok(dir) = entry {
            match Repository::try_from(dir.path()) {
                Ok(mut repo) => match repo
                    .parse_profiles()
                    .with_context(|| format!("Failed while parsing profiles of {}", &repo.name))
                {
                    Ok(_) => {
                        let mut failed = false;
                        for p in repo.profiles.values_mut() {
                            match p.parse_and_ingest_conf().with_context(|| {
                                format!(
                                    "Failed while parsing {}",
                                    dir.path().join(&p.name).display()
                                )
                            }) {
                                Ok(_) => continue,
                                Err(e) => {
                                    eprintln!("Warning: skipping repository {}: {}", repo.name, e);
                                    failed = true;
                                    break;
                                }
                            }
                        }
                        if !failed {
                            self.map.insert(repo.name.clone(), repo);
                        }
                        WalkState::Skip
                    }
                    Err(e) => {
                        eprintln!("Warning: skipping repository at {}: {}", dir.path().display(), e);
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

        let mut table = RepositoryTableBuilder::new();

        let walker = walker.build_parallel();

        walker.visit(&mut table);

        let table = RepositoryTable::try_from(table)?;

        // The table should have exactly 3 repositories: `spam`, `ham`, and `eggs`.
        assert_eq!(table.map.len(), 3);

        Ok(())
    }

    #[test]
    fn test_table_builder_skips_bad_make_default() {
        let test_tree_dir = test_data_dir(&["broken-test-tree"]);

        let mut walker = ignore::WalkBuilder::new(&test_tree_dir);
        walker.filter_entry(|dir| dir.path().is_dir());

        let mut table = RepositoryTableBuilder::new();

        let walker = walker.build_parallel();
        walker.visit(&mut table);

        // Repository with bad make.defaults is skipped, not fatal.
        let table = RepositoryTable::try_from(table).unwrap();
        assert_eq!(table.map.len(), 0);
    }
}
