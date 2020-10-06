use anyhow;
use clap::ArgMatches;

use crate::graph;
use crate::parse;
use crate::portage::{overlay::build_overlay_map, ProfileKey};

pub fn parents(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let targets = sub_args.values_of("profile").unwrap();
    let targets: Vec<_> = targets
        .flat_map(|target| parse::parse_parent_file(&target))
        .collect();

    let overlay_table = build_overlay_map(&config)?;

    let start_profiles: Vec<ProfileKey> = targets
        .into_iter()
        .flat_map(|v| v.into_iter())
        .map(|(r, p)| ProfileKey::new(r.unwrap(), p.to_string_lossy()))
        .collect();

    if sub_args.is_present("tree") {
        // print_profile_tree(0, &profile, &profile_map);
        todo!();
    }

    if sub_args.is_present("graph") {
        graph::dump_graphviz(&overlay_table, &start_profiles);
    }

    Ok(())
}

pub fn eval(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("profile").unwrap();
    let profile = parse::parse_parent_file(&target)
        .unwrap()
        .into_iter()
        .map(|(o, p)| ProfileKey::new(o.unwrap(), p.to_string_lossy()))
        .next()
        .unwrap();

    let target_var = sub_args.value_of("variable").unwrap();
    let overlay_table = build_overlay_map(&config)?;

    println!("{}", overlay_table.get_var(&profile, target_var)?);
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
