// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

mod commands;
mod config;
mod graph;
mod parse;
mod portage;

#[cfg(test)]
mod test_util;

use clap::{Arg, ArgAction, Command};

fn main() -> anyhow::Result<()> {
    let args = Command::new("drydock")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for Portage profile analysis and introspection.")
        .after_help("Tip: The full `--help` flag gives more verbose explanations of options.")
        .arg_required_else_help(true)
        .arg(
            Arg::new("config_file")
                .long("config-file")
                .help("Path to the configuration file to use.")
                .global(true),
        )
        .arg(
            Arg::new("src_path")
                .long("src-path")
                .help("Path to the directory containing your repositories.")
                .long_help(
                    "Path to the directory containing your repositories. Specifying this \
                    on the command line overrides the value found in the configuration file. \
                    Typically /var/db/repos.",
                )
                .global(true),
        )
        .subcommand(
            Command::new("parents")
                .about("Show the inheritance tree for the target profile; prints an indented tree by default.")
                .arg(
                    Arg::new("profile")
                        .required(true)
                        .num_args(1..)
                        .help("The target profile. Example: gentoo:default/linux/amd64/23.0"),
                )
                .arg(
                    Arg::new("graph")
                        .long("graph")
                        .action(ArgAction::SetTrue)
                        .help("Print graphviz dot formatting for the profile parent structure."),
                ),
        )
        .subcommand(
            Command::new("eval")
                .about("Print the final value of a config variable for a profile.")
                .arg(
                    Arg::new("profile")
                        .short('p')
                        .long("profile")
                        .required(true)
                        .help("The target profile to query."),
                )
                .arg(
                    Arg::new("variable")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("blame")
                .about(
                    "Show the value of a variable for a profile annotated with the sources \
                        of that variable's contents.",
                )
                .arg(
                    Arg::new("profile")
                        .short('p')
                        .long("profile")
                        .required(true)
                        .help("The target profile to query."),
                )
                .arg(
                    Arg::new("variable")
                        .required(true)
                        .num_args(1..=2)
                        .value_delimiter(':')
                        .value_names(["VARIABLE", "TOKEN"]),
                ),
        )
        .subcommand(
            Command::new("dump_debug")
                .about("Dump debug information for a repository.")
                .arg(
                    Arg::new("repository")
                        .short('r')
                        .long("repository")
                        .required(true)
                        .help("The target repository to query."),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Set configuration values for drydock.")
                .arg(
                    Arg::new("default")
                        .long("default")
                        .action(ArgAction::SetTrue)
                        .required(true)
                        .help("Generate a default configuration file."),
                ),
        )
        .get_matches();

    if let Some(("config", _)) = args.subcommand() {
        crate::config::generate_default(
            args.get_one::<String>("config_file").map(|s| s.as_str()),
            args.get_one::<String>("src_path").map(|s| s.as_str()),
        )?
    }

    let config = crate::config::DrydockConfig::load(
        args.get_one::<String>("config_file").map(|s| s.as_str()),
        args.get_one::<String>("src_path").map(|s| s.as_str()),
    )?;

    match args.subcommand() {
        Some(("config", _)) => {}
        Some(("blame", sub_args)) => commands::blame(&config, sub_args)?,
        Some(("dump_debug", sub_args)) => commands::dump_debug(&config, sub_args)?,
        Some(("eval", sub_args)) => commands::eval(&config, sub_args)?,
        Some(("parents", sub_args)) => commands::parents(&config, sub_args)?,
        _ => unreachable!(),
    };

    Ok(())
}
