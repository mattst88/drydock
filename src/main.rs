mod commands;
mod graph;
mod parse;
mod portage;

use std::env;

use anyhow;
use clap::{App, Arg, SubCommand};
use config;

fn main() -> anyhow::Result<()> {
    let config_path = env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME").unwrap() + "/.config")
        + "/drydock/config.toml";
    let mut config = config::Config::new();
    config.merge(config::File::with_name(&config_path)).unwrap();

    let args = App::new("drydock")
        .version("0.0.1")
        .about("A tool for Portage profile analysis and introspection.")
        .subcommand(
            SubCommand::with_name("parents")
                .about("Show the inheritance tree for the target profile.")
                .arg(
                    Arg::with_name("profile")
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .help("The target profile. Example: chromiumos:base"),
                )
                .arg(
                    Arg::with_name("tree")
                    .long("tree")
                    .takes_value(false)
                    .help("Print an indented tree of the profile parent structure.")
                )
                .arg(
                    Arg::with_name("graph")
                    .long("graph")
                    .takes_value(false)
                    .help("Print graphviz dot formatting for the profile parent structure.")
                ),
        )
        .get_matches();

    if let Some(sub_args) = args.subcommand_matches("parents") {
        commands::parents(&config, sub_args)
    } else {
        unimplemented!()
    }
}
