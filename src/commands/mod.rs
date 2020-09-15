use anyhow;
use clap::ArgMatches;

use crate::graph;
use crate::parse;
use crate::portage::{overlay::build_overlay_map, profile::explore, Overlay, Profile};

use std::collections::HashMap;

pub fn parents(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let targets = sub_args.values_of("profile").unwrap();
    let targets: Vec<_> = targets
        .flat_map(|target| parse::parse_parent_file(&target))
        .collect();

    let overlay_map: HashMap<String, Overlay> = build_overlay_map(&config);

    let mut profile_map: HashMap<Profile, Vec<Profile>> = HashMap::new();

    let start_profiles: Vec<Profile> = targets
        .into_iter()
        .map(|(r, p)| overlay_map[&r.unwrap()].profile_from(p))
        .collect::<Result<Vec<_>, _>>()?;

    for profile in start_profiles {
        explore(profile.clone(), &mut profile_map, &overlay_map)?;

        if sub_args.is_present("tree") { print_profile_tree(0, &profile, &profile_map); }
    }

    if sub_args.is_present("graph") { graph::dump_graphviz(&profile_map); }

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
