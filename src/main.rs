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

#[macro_use]
extern crate rental;

use clap::{App, Arg, SubCommand};

fn main() -> anyhow::Result<()> {
    let args = App::new("drydock")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A tool for Portage profile analysis and introspection.")
        .after_help("Tip: The full `--help` flag gives more verbose explanations of options.")
        .arg(
            Arg::with_name("config_file")
                .long("config-file")
                .takes_value(true)
                .help("Path to the configuration file to use.")
                .global(true),
        )
        .arg(
            Arg::with_name("src_path")
                .long("src-path")
                .takes_value(true)
                .help("Path to the directory containing your repositories.")
                .long_help(
                    "Path to the directory containing your repositories. Specifying this \
                    on the command line overrides the value found in the configuration file. \
                    Typically /var/db/repos.",
                )
                .global(true),
        )
        .subcommand(
            SubCommand::with_name("parents")
                .about("Show the inheritance tree for the target profile; prints an indented tree by default.")
                .arg(
                    Arg::with_name("profile")
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .help("The target profile. Example: gentoo:default/linux/amd64/23.0"),
                )
                .arg(
                    Arg::with_name("graph")
                        .long("graph")
                        .takes_value(false)
                        .help("Print graphviz dot formatting for the profile parent structure."),
                ),
        )
        .subcommand(
            SubCommand::with_name("eval")
                .about("Print the final value of a config variable for a profile.")
                .arg(
                    Arg::with_name("profile")
                        .short("p")
                        .long("profile")
                        .takes_value(true)
                        .required(true)
                        .help("The target profile to query."),
                )
                .arg(
                    Arg::with_name("variable")
                        .takes_value(true)
                        .required(true)
                        .multiple(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("blame")
                .about(
                    "Show the value of a variable for a profile annotated with the sources \
                        of that variable's contents.",
                )
                .arg(
                    Arg::with_name("profile")
                        .short("p")
                        .long("profile")
                        .takes_value(true)
                        .required(true)
                        .help("The target profile to query."),
                )
                .arg(
                    Arg::with_name("variable")
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .min_values(1)
                        .max_values(2)
                        .value_delimiter(":")
                        .require_delimiter(true)
                        .value_names(&["VARIABLE", "TOKEN"]),
                ),
        )
        .subcommand(
            SubCommand::with_name("dump_debug")
                .about("Dump debug information for a repository.")
                .arg(
                    Arg::with_name("repository")
                        .short("r")
                        .long("repository")
                        .takes_value(true)
                        .required(true)
                        .help("The target repository to query."),
                ),
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Set configuration values for drydock.")
                .arg(
                    Arg::with_name("default")
                        .long("default")
                        .takes_value(false)
                        .required(true)
                        .help("Generate a default configuration file."),
                ),
        )
        .get_matches();

    if let ("config", _) = args.subcommand() {
        crate::config::generate_default(args.value_of("config_file"), args.value_of("src_path"))?
    }

    let config = crate::config::DrydockConfig::load(
        args.value_of("config_file"),
        args.value_of("src_path"),
    )?;

    match args.subcommand() {
        ("config", _) => {}
        ("blame", Some(sub_args)) => commands::blame(&config, sub_args)?,
        ("dump_debug", Some(sub_args)) => commands::dump_debug(&config, sub_args)?,
        ("eval", Some(sub_args)) => commands::eval(&config, sub_args)?,
        ("parents", Some(sub_args)) => commands::parents(&config, sub_args)?,
        _ => println!("{}", args.usage()),
    };

    Ok(())
}
