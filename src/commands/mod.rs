use std::fmt::Write;
use std::{collections::HashSet, str::FromStr};

use clap::ArgMatches;

use crate::{graph, portage::profile::is_incremental_variable, portage::profile_parser::Span};

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
    if is_incremental_variable(target_var) {
        let vals = overlay_table.compute_variable(&profile, target_var)?;
        let mut token_set = HashSet::new();

        for val in vals.iter() {
            for token in val.fragment().split_ascii_whitespace() {
                if token.starts_with('-') {
                    token_set.remove(&token.strip_prefix('-').unwrap());
                } else {
                    token_set.insert(token);
                }
            }
        }

        let mut tokens: Vec<&str> = token_set.into_iter().collect();
        tokens.sort_unstable();
        tokens.dedup();

        println!("{}", tokens.as_slice().join(" "))
    } else {
        let vals = overlay_table.compute_variable(&profile, target_var)?;
        println!("{}", simple_format(&vals));
    }
    Ok(())
}

pub fn dump_debug(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("overlay").unwrap();
    let overlay_table = build_overlay_map(&config)?;

    println!("{:#?}", overlay_table.map[target]);
    Ok(())
}

fn simple_format(tokens: &[Span]) -> String {
    let mut output = String::new();

    for token in tokens {
        write!(&mut output, "{}", token.fragment()).unwrap();
    }
    output
}
