use std::{collections::HashSet, iter::Rev, slice::Iter};

use petgraph::visit::{Data, DfsPostOrder, IntoNeighbors, Visitable};
use petgraph::{data::DataMap, visit::GraphBase};

use crate::portage::{Profile, ProfileKey};

use super::OverlayTable;

impl<'a> GraphBase for &'a OverlayTable {
    type EdgeId = (&'a ProfileKey, &'a ProfileKey);
    type NodeId = &'a ProfileKey;
}

impl<'a> Data for &'a OverlayTable {
    type NodeWeight = Profile;
    type EdgeWeight = ();
}

impl<'a> DataMap for &'a OverlayTable {
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

impl<'a> Visitable for &'a OverlayTable {
    type Map = HashSet<Self::NodeId>;

    fn visit_map(&self) -> Self::Map {
        Self::Map::new()
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear()
    }
}

impl<'a: 'b, 'b> IntoNeighbors for &'b &'a OverlayTable {
    type Neighbors = Iter<'a, ProfileKey>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.get(a).map(|p| p.parents.iter()).unwrap_or([].iter())
    }
}

pub(super) struct ProfileIter<'a> {
    visitor:
        DfsPostOrder<<&'a OverlayTable as GraphBase>::NodeId, <&'a OverlayTable as Visitable>::Map>,
    overlay_table: &'a OverlayTable,
}

impl<'a> ProfileIter<'a> {
    pub(super) fn new(
        overlay_table: &'a OverlayTable,
        start: <&'a OverlayTable as GraphBase>::NodeId,
    ) -> Self {
        Self {
            overlay_table,
            visitor: DfsPostOrder::new(&overlay_table, start),
        }
    }
}

impl<'a> Iterator for ProfileIter<'a> {
    type Item = <&'a OverlayTable as GraphBase>::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.visitor.next(&self.overlay_table)
    }
}
