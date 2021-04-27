//! Module containing the implementations of each subcommand.
//! Subcommands all have the same function signature, accepting a &[crate::config::DrydockConfig],
//! an [clap::ArgMatches], and returning a [anyhow::Result<()>].

mod blame;

pub use blame::blame;

use std::fmt::Write;
use std::str::FromStr;

use clap::ArgMatches;

use crate::{
    config::DrydockConfig,
    graph,
    portage::profile_parser::Span,
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
    if overlay_table.is_incremental_variable(&profile, target_var) {
        let vals = overlay_table.compute_variable(&profile, target_var)?;

        let mut output = String::new();
        for val in vals {
            write!(&mut output, "{} ", val.fragment()).unwrap();
        }
        println!("{}", output);
    } else {
        let vals = overlay_table.compute_variable(&profile, target_var)?;
        println!("{}", simple_format(&vals));
    }
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

/// Helper function to flatten a slice of [Span]s into a single [String].
/// [Span]s are essentially just a wrapper around &[str] with some additional metadata.
fn simple_format(tokens: &[Span]) -> String {
    let mut output = String::new();

    for token in tokens {
        write!(&mut output, "{}", token.fragment()).unwrap();
    }
    output
}
