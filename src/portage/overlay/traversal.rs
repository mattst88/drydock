use std::collections::HashSet;

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
    fn node_weight(self: &Self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.get(id)
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
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

    fn visit_map(self: &Self) -> Self::Map {
        Self::Map::new()
    }

    fn reset_map(self: &Self, map: &mut Self::Map) {
        map.clear()
    }
}

impl<'a: 'b, 'b> IntoNeighbors for &'b &'a OverlayTable {
    type Neighbors = std::slice::Iter<'a, ProfileKey>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        self.get(a).map(|p| p.parents.iter()).unwrap_or([].iter())
    }
}

struct ProfileIter<'a> {
    visitor:
        DfsPostOrder<<&'a OverlayTable as GraphBase>::NodeId, <&'a OverlayTable as Visitable>::Map>,
    overlay_table: &'a OverlayTable,
}

impl<'a> Iterator for ProfileIter<'a> {
    type Item = <&'a OverlayTable as GraphBase>::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.visitor.next(&self.overlay_table)
    }
}
