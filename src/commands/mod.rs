//! Module containing the implementations of each subcommand.
//! Subcommands all have the same function signature, accepting a &[crate::config::DrydockConfig],
//! an [clap::ArgMatches], and returning a [anyhow::Result<()>].

mod blame;

pub use blame::blame;

use std::str::FromStr;

use clap::ArgMatches;

use crate::{
    config::DrydockConfig,
    graph,
    portage::{overlay::build_overlay_map, ProfileKey},
};

pub fn parents(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let targets = sub_args.values_of("profile").unwrap();
    let targets: Vec<_> = targets
        .map(|target| ProfileKey::from_str(target).unwrap())
        .collect();

    let overlay_table = build_overlay_map(config)?;

    if sub_args.is_present("tree") {
        for target in targets.iter() {
            overlay_table.print_profile_tree(std::io::stdout(), target)?;
        }
    }

    if sub_args.is_present("graph") {
        graph::dump_graphviz(std::io::stdout(), &overlay_table, &targets)?;
    }

    Ok(())
}

/// Evaluate a Portage variable and print the contents to stdout.
pub fn eval(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("profile").unwrap();
    let profile = ProfileKey::from_str(target)?;

    let target_var = sub_args.value_of("variable").unwrap();
    let overlay_table = build_overlay_map(config)?;

    let vals = overlay_table.compute_variable(&profile, target_var)?;
    let output: String = vals.into_iter().map(|s| *s).collect();
    println!("{}", output);

    Ok(())
}

/// Dump an ugly Debug representation of an [crate::portage::overlay::Overlay] to aid in manual
/// debugging of behavior.
pub fn dump_debug(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("overlay").unwrap();
    let overlay_table = build_overlay_map(&config)?;

    println!("{:#?}", overlay_table.map[target]);
    Ok(())
}
