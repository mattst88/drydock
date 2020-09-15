use std::collections::{HashMap, HashSet};

use petgraph::graphmap::DiGraphMap;
use petgraph::dot;

use crate::overlay::Profile;

pub fn dump_graphviz(profile_map: &HashMap<Profile, Vec<Profile>>) {
    let mut qualified_profile_names: HashSet<String> = HashSet::new();

    for p in profile_map.keys() {
        qualified_profile_names.insert(format!("{}:{}", p.overlay.name, p.rel_path.display()));
    }

    for p in profile_map.values().flat_map(|item| item.iter()) {
        qualified_profile_names.insert(format!("{}:{}", p.overlay.name, p.rel_path.display()));
    }

    let mut graphmap = DiGraphMap::new();

    for (k, v) in profile_map {
        let key_name = format!("{}:{}", k.overlay.name, k.rel_path.display());

        for v in v {
            let value_name = format!("{}:{}", v.overlay.name, v.rel_path.display());

            graphmap.add_edge(
                qualified_profile_names.get(&key_name).unwrap(),
                qualified_profile_names.get(&value_name).unwrap(),
                (),
            );
        }
    }

    println!("{:?}", dot::Dot::with_config(&graphmap, &[dot::Config::EdgeNoLabel]));

}
