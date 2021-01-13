mod commands;
mod config;
mod graph;
mod parse;
mod portage;

#[macro_use]
extern crate rental;

use clap::{App, Arg, SubCommand};

fn main() -> anyhow::Result<()> {
    let args = App::new("drydock")
        .version("0.0.3")
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
                        .help("Print an indented tree of the profile parent structure."),
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
                .about("Show the value of a variable for a profile annotated with the sources of that variable's contents.")
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
                .about("Dump debug information for an overlay.")
                .arg(
                    Arg::with_name("overlay")
                        .short("o")
                        .long("overlay")
                        .takes_value(true)
                        .required(true)
                        .help("The target overlay to query."),
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
        crate::config::generate_default()?
    }

    let config = crate::config::get()?;

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
