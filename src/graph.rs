// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{collections::HashSet, io};

use anyhow::bail;
use petgraph::dot;
use petgraph::graphmap::DiGraphMap;

use crate::portage::repository::RepositoryTable;
use crate::portage::profile::ProfileKey;

/// Helper function to print the graph ancestors of a profile in graphviz's DOT format.
/// Currently operates by traversing the [RepositoryTable] and adding the profile & parents
/// to a [DiGraphMap] and using petgraph's builtin DOT formatter.
pub fn dump_graphviz(
    mut dest: impl io::Write,
    table: &RepositoryTable,
    roots: &[ProfileKey],
) -> anyhow::Result<()> {
    let mut graphmap: DiGraphMap<&str, (), std::hash::RandomState> = DiGraphMap::default();

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

        if let Some(o) = table.map.get(key.repo_name()) {
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
                "Missing a repository!\n Requested: {:?}\nVisited: {:?}\nKnown: {:?}",
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

    use crate::{config::DrydockConfig, portage::repository::build_repository_table};

    use crate::test_util::test_data_dir;

    #[test]
    fn test_graphviz_basic_test_tree() -> anyhow::Result<()> {
        let test_tree = test_data_dir(&["test-tree"]);

        let config = DrydockConfig {
            src_path: test_tree,
            ..Default::default()
        };
        let roots = &[ProfileKey::new("ham", "base")];

        let repo_table = build_repository_table(&config)?;

        let mut buf = Vec::new();
        dump_graphviz(&mut buf, &repo_table, roots)?;

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
