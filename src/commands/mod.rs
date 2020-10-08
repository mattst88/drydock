use std::str::FromStr;

use clap::ArgMatches;

use crate::graph;

use crate::portage::{overlay::build_overlay_map, ProfileKey};

pub fn parents(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let targets = sub_args.values_of("profile").unwrap();
    let targets: Vec<_> = targets
        .map(|target| ProfileKey::from_str(target).unwrap())
        .collect();

    let overlay_table = build_overlay_map(&config)?;

    if sub_args.is_present("tree") {
        // print_profile_tree(0, &profile, &profile_map);
        todo!();
    }

    if sub_args.is_present("graph") {
        graph::dump_graphviz(&overlay_table, &targets);
    }

    Ok(())
}

pub fn eval(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("profile").unwrap();
    let profile = ProfileKey::from_str(target)?;

    let target_var = sub_args.value_of("variable").unwrap();
    let overlay_table = build_overlay_map(&config)?;

    println!("{}", overlay_table.compute_variable(&profile, target_var)?);
    Ok(())
}

pub fn dump_debug(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("overlay").unwrap();
    let overlay_table = build_overlay_map(&config)?;

    println!("{:#?}", overlay_table.map[target]);
    Ok(())
}
// fn print_profile_tree<'a>(
//     depth: usize,
//     profile: &'a Profile<'a>,
//     profile_map: &'a HashMap<Profile<'a>, Vec<Profile<'a>>>,
// ) {
//     for _ in 0..depth {
//         print!("  ");
//     }
//     println!("{}:{}", profile.overlay.name, profile.rel_path.display());
//     for p in profile_map.get(profile).unwrap_or(&Vec::new()).iter() {
//         print_profile_tree(depth + 1, p, profile_map)
//     }
// }
