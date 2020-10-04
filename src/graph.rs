use std::collections::{HashMap, HashSet};

use petgraph::dot;
use petgraph::graphmap::DiGraphMap;

use crate::portage::overlay::OverlayTable;
use crate::portage::profile::ProfileKey;

pub fn dump_graphviz(table: &OverlayTable, roots: &[ProfileKey]) {
    let mut graphmap: DiGraphMap<&str, ()> = DiGraphMap::new();

    for root in roots {
        graphmap.add_node(root.full_name());
    }

    let mut frontier: Vec<&ProfileKey> = roots.iter().collect();
    let mut visited: HashSet<ProfileKey> = HashSet::new();

    while let Some(key) = frontier.pop() {
        if visited.contains(key) {
            continue;
        } else {
            visited.insert(key.clone());
        }

        if let Some(o) = table.map.get(key.overlay()) {
            if let Some(p) = o.profiles.get(key.profile()) {
                for parent in p.parents.iter() {
                    graphmap.add_edge(key.full_name(), parent.full_name(), ());
                    frontier.push(parent)
                }
            } else {
                eprintln!("{} : {}", key.overlay(), key.profile());
                panic!("Missing a profile!\n Requested: {:?}\nFound: {:?}", key, o);
            }
        } else {
            eprintln!("{} : {}", key.overlay(), key.profile());
            let mut keys: Vec<String> = table.map.keys().cloned().collect();
            keys.sort();
            panic!("Missing an overlay!\n Requested: {:?}\n", key);
        }
    }

    println!(
        "{:?}",
        dot::Dot::with_config(&graphmap, &[dot::Config::EdgeNoLabel])
    );
}
