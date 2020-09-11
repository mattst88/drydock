use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// use petgraph;

const PARENT_FILE: &'static str = "parent";

fn main() {
    let target = dbg!(PathBuf::from(env::args_os().nth(1).unwrap()));

    let mut ancestors = target.ancestors();
    let profile_root = ancestors.find(|p| p.ends_with("profiles")).unwrap();

    let mut profile_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let rel_path = target.strip_prefix(profile_root).unwrap().to_owned();

    explore(rel_path, &mut profile_map, profile_root);
    dbg!(profile_map);
}

fn explore(
    rel_path: PathBuf,
    profile_map: &mut HashMap<PathBuf, Vec<PathBuf>>,
    profile_root: &Path,
) {
    let mut parents: Vec<PathBuf> = Vec::new();

    let target = dbg!(profile_root.join(&rel_path));
    if let Ok(parent_file) = fs::read_to_string(target.join(PARENT_FILE)) {
        for line in parent_file.lines() {
            if line.trim().is_empty() {
                continue;
            };
            parents.push(
                target
                    .join(line.trim())
                    .canonicalize()
                    .unwrap()
                    .strip_prefix(profile_root)
                    .unwrap()
                    .to_owned(),
            );
        }

        profile_map.insert(rel_path, parents.clone());
        for p in &parents {
            explore(p.clone(), profile_map, profile_root)
        }
    }
}
