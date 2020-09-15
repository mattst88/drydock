mod overlay;
mod parse;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs;
use std::path::PathBuf;

use overlay::{Overlay, Profile};

use anyhow;
use config;
use ignore;
// use petgraph;

const PARENT_FILE: &'static str = "parent";

fn main() -> anyhow::Result<()> {
    let config_path = env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME").unwrap() + "/.config")
        + "/drydock/config.toml";
    let mut settings = config::Config::new();
    settings
        .merge(config::File::with_name(&config_path))
        .unwrap();

    let target = dbg!(env::args().nth(1).unwrap());
    let (repo, rel_path) = dbg!(parse::parse_parent_file(&target)
        .into_iter()
        .nth(0)
        .unwrap());

    let overlay_map: HashMap<String, Overlay> = build_overlay_map(&settings);

    let mut profile_map: HashMap<Profile, Vec<Profile>> = HashMap::new();

    let start_profile = overlay_map[&repo.unwrap()].profile_from(rel_path).unwrap();

    explore(start_profile.clone(), &mut profile_map, &overlay_map)?;

    print_profile_tree(0, &start_profile, &profile_map);
    Ok(())
}

fn explore<'a>(
    profile: Profile<'a>,
    profile_map: &mut HashMap<Profile<'a>, Vec<Profile<'a>>>,
    overlay_map: &'a HashMap<String, Overlay>,
) -> anyhow::Result<()> {
    if let Ok(parent_file) = fs::read_to_string(profile.full_path().join(PARENT_FILE)) {
        for (overlay_name, raw_path) in parse::parse_parent_file(&parent_file) {
            let new_profile: Profile = match overlay_name {
                Some(overlay_name) => {
                    let target_overlay = overlay_map.get(&overlay_name).unwrap();
                    target_overlay.profile_from(raw_path)?
                }
                None => profile.create_relative(raw_path)?,
            };

            let parent_list = profile_map.entry(profile.clone()).or_insert(Vec::new());
            if parent_list.iter().find(|p| p == &&new_profile).is_none() {
                parent_list.push(new_profile);
            }
        }
        let frontier = profile_map.get(&profile).cloned().unwrap_or_default();
        for p in frontier {
            explore(p, profile_map, overlay_map)?
        }
    }
    Ok(())
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

fn build_overlay_map(config: &config::Config) -> HashMap<String, Overlay> {
    let mut walker = ignore::WalkBuilder::new(".");
    walker.max_depth(Some(1));

    for overlay_path in config.get_array("overlay_paths").unwrap() {
        let p = dbg!(overlay_path.into_str().unwrap());
        walker.add(p);
    }

    let mut map = HashMap::new();

    for candidate_path in walker.build() {
        let candidate_path = candidate_path.unwrap();
        if let Ok(x) = Overlay::try_from(candidate_path.path()) {
            map.insert(x.name.clone(), x);
        }
    }

    map
}
