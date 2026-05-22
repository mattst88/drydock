// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! A module containing trait impls for compatibility with petgraph.
//!
//! For the purpose of these traits An [RepositoryTable] is treated as a directed graph
//! where the nodes are [Profile]s and a directed edge from A to B exists if B is
//! listed in `parent` file of A.

use std::{collections::HashSet, slice::Iter};

use petgraph::visit::{Data, DfsPostOrder, IntoNeighbors, Visitable};
use petgraph::{data::DataMap, visit::GraphBase};

use crate::portage::{Profile, ProfileKey};

use super::RepositoryTable;

impl<'a> GraphBase for &'a RepositoryTable {
    type EdgeId = (&'a ProfileKey, &'a ProfileKey);
    type NodeId = &'a ProfileKey;
}

impl<'a> Data for &'a RepositoryTable {
    type NodeWeight = Profile;
    type EdgeWeight = ();
}

impl<'a> DataMap for &'a RepositoryTable {
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.get(id)
    }

    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.get(id.0).and_then(|p| {
            if p.parents.contains(id.1) {
                Some(&())
            } else {
                None
            }
        })
    }
}

impl<'a> Visitable for &'a RepositoryTable {
    type Map = HashSet<Self::NodeId>;

    fn visit_map(&self) -> Self::Map {
        Self::Map::new()
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear()
    }
}

impl<'a: 'b, 'b> IntoNeighbors for &'b &'a RepositoryTable {
    type Neighbors = Iter<'a, ProfileKey>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.get(a)
            .map(|p| p.parents.iter())
            .unwrap_or_else(|| [].iter())
    }
}

pub(super) struct ProfileIter<'a> {
    visitor:
        DfsPostOrder<<&'a RepositoryTable as GraphBase>::NodeId, <&'a RepositoryTable as Visitable>::Map>,
    repo_table: &'a RepositoryTable,
}

impl<'a> ProfileIter<'a> {
    #[allow(dead_code)]
    pub(super) fn new(
        repo_table: &'a RepositoryTable,
        start: <&'a RepositoryTable as GraphBase>::NodeId,
    ) -> Self {
        Self {
            repo_table,
            visitor: DfsPostOrder::new(&repo_table, start),
        }
    }
}

impl<'a> Iterator for ProfileIter<'a> {
    type Item = <&'a RepositoryTable as GraphBase>::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.visitor.next(&self.repo_table)
    }
}
