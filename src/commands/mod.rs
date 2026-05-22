// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

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
    portage::{repository::build_repository_table, ProfileKey},
};

/// Print the inheritance hierarchy of a given Portage profile.
///
/// Can print a simple textual representation of an inheritance tree or a DOT representation
/// suitable for rendering via `graphviz`.
pub fn parents(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let targets: Vec<_> = sub_args
        .get_many::<String>("profile")
        .unwrap()
        .map(|target| ProfileKey::from_str(target).unwrap())
        .collect();

    let repo_table = build_repository_table(config)?;

    if sub_args.get_flag("graph") {
        graph::dump_graphviz(std::io::stdout(), &repo_table, &targets)?;
    } else {
        for target in targets.iter() {
            repo_table.print_profile_tree(std::io::stdout(), target)?;
        }
    }

    Ok(())
}

/// Evaluate a Portage variable and print the contents to stdout.
pub fn eval(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.get_one::<String>("profile").unwrap().as_str();
    let profile = ProfileKey::from_str(target)?;

    let target_var = sub_args.get_one::<String>("variable").unwrap().as_str();
    let repo_table = build_repository_table(config)?;

    let vals = repo_table.compute_variable(&profile, target_var)?;

    let mut output = String::new();
    vals.into_iter()
        .try_for_each(|s| write!(&mut output, "{} ", *s))?;

    println!("{}", output.trim_end());

    Ok(())
}

/// Dump an ugly Debug representation of a [crate::portage::repository::Repository] to aid in manual
/// debugging of behavior.
pub fn dump_debug(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.get_one::<String>("repository").unwrap().as_str();
    let repo_table = build_repository_table(&config)?;

    println!("{:#?}", repo_table.map[target]);
    Ok(())
}
