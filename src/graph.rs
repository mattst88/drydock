// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{collections::HashSet, io};

use anyhow::bail;
use petgraph::dot;
use petgraph::graphmap::DiGraphMap;

use crate::portage::overlay::OverlayTable;
use crate::portage::profile::ProfileKey;

/// Helper function to print the graph ancestors of a profile in graphviz's DOT format.
/// Currently operates by traversing the [OverlayTable] and adding the profile & parents
/// to a [DiGraphMap] and using petgraph's builtin DOT formatter.
pub fn dump_graphviz(
    mut dest: impl io::Write,
    table: &OverlayTable,
    roots: &[ProfileKey],
) -> anyhow::Result<()> {
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
                bail!("Missing a profile!\n Requested: {:?}\nFound: {:?}", key, o);
            }
        } else {
            let mut keys: Vec<String> = table.map.keys().cloned().collect();
            keys.sort();
            bail!(
                "Missing an overlay!\n Requested: {:?}\nVisited: {:?}\nKeys: {:?}",
                key,
                visited,
                keys
            );
        }
    }

    writeln!(
        &mut dest,
        "{:?}",
        dot::Dot::with_config(&graphmap, &[dot::Config::EdgeNoLabel])
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::{Path, PathBuf};

    use crate::{config::DrydockConfig, portage::overlay::build_overlay_map};

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
    fn test_graphviz_basic_test_tree() -> anyhow::Result<()> {
        let test_tree = test_data_dir(&["test-tree"]);

        let config = DrydockConfig {
            src_path: test_tree,
            ..Default::default()
        };
        let roots = &[ProfileKey::new("ham", "base")];

        let overlay_table = build_overlay_map(&config)?;

        let mut buf = Vec::new();
        dump_graphviz(&mut buf, &overlay_table, roots)?;

        let output = String::from_utf8(buf)?;

        // This output is deterministic since there are only two nodes: The node in
        // the `roots` argument will always be first.
        assert_eq!(
            output,
            r#"digraph {
    0 [ label = "\"ham:base\"" ]
    1 [ label = "\"eggs:base\"" ]
    0 -> 1 [ ]
}

"#
        );

        Ok(())
    }
}
