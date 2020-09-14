mod overlay;
mod parse;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use overlay::{Overlay, Profile};
// use petgraph;

const PARENT_FILE: &'static str = "parent";

fn main() {
    let target = dbg!(PathBuf::from(env::args_os().nth(1).unwrap()));

    let mut ancestors = target.ancestors();
    let profile_root = ancestors.find(|p| p.ends_with("profiles")).unwrap();
    let overlay = Overlay::new(
        "chromiumos".to_owned(),
        profile_root.parent().unwrap().to_owned(),
    );

    let mut overlay_map: HashMap<String, Overlay> = HashMap::new();
    overlay_map.insert(overlay.name.clone(), overlay.clone());

    let mut profile_map: HashMap<Profile, Vec<Profile>> = HashMap::new();
    let start_profile = overlay
        .profile_from(target.strip_prefix(profile_root).unwrap().to_owned())
        .unwrap();

    explore(start_profile.clone(), &mut profile_map, &overlay_map);

    print_profile_tree(0, &start_profile, &profile_map);
}

fn explore<'a>(
    profile: Profile<'a>,
    profile_map: &mut HashMap<Profile<'a>, Vec<Profile<'a>>>,
    overlay_map: &'a HashMap<String, Overlay>,
) {
    if let Ok(parent_file) = fs::read_to_string(profile.full_path().join(PARENT_FILE)) {
        for (overlay_name, raw_path) in parse::parse_parent_file(&parent_file) {
            let new_profile: Profile = match overlay_name {
                Some(overlay_name) => {
                    let target_overlay = overlay_map.get(&overlay_name).unwrap();
                    target_overlay.profile_from(raw_path).unwrap()
                }
                None => profile.create_relative(raw_path).unwrap(),
            };

            let parent_list = profile_map.entry(profile.clone()).or_insert(Vec::new());
            parent_list.push(new_profile);
        }
        let frontier = profile_map.get(&profile).cloned().unwrap_or_default();
        for p in frontier {
            explore(p, profile_map, overlay_map)
        }
    }
}

fn print_profile_tree<'a>(
    depth: usize,
    profile: &'a Profile<'a>,
    profile_map: &'a HashMap<Profile<'a>, Vec<Profile<'a>>>,
) {
    for _ in 0..depth {
        print!("  ");
    }
    println!("{}:{}", profile.overlay.name, profile.rel_path.display());
    for p in profile_map.get(profile).unwrap_or(&Vec::new()).iter() {
        print_profile_tree(depth + 1, p, profile_map)
    }
}
