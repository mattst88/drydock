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
