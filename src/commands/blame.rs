use std::cmp::max;
use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::str::FromStr;

use anyhow::bail;
use clap::ArgMatches;
use colored::Colorize;
use source_span::{fmt::Style, Position};

use crate::{
    config::DrydockConfig,
    portage::{
        overlay::build_overlay_map, profile_parser::Span, variables::TokenState, ProfileKey,
    },
};

/// Print the parsed syntax tree for the contents of a variable with the annotated location
/// of each leaf in the tree.
///
/// Portage really has two types of variables: incremental and non-incremental.
/// Non-incremental variables behave like familiar variables in other settings: the most
/// recent definition is the one that is used. Incremental variables function more like sets
/// of tokens, with each node in the inheritance tree having a separate definition of that
/// variable and the final value being the concatenated union of every profile's definition
/// of that variable.
///
/// This function currently behaves very differently depending on whether or not the specified
/// variable is incremental or not, but in the interest of not burdening the user with having
/// to understand the distinction we handle both cases in this command.
pub fn blame(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("profile").unwrap();
    let profile = ProfileKey::from_str(target)?;

    let mut target_values = sub_args.values_of("variable").unwrap();
    let target_var = target_values.next().unwrap();
    let overlay_table = build_overlay_map(config)?;
    if overlay_table.is_incremental_variable(&profile, target_var) {
        if let Some(subvar) = target_values.next() {
            let sets = overlay_table.compute_incremental_variable(&profile, target_var)?;
            let name_width = sets
                .iter()
                .map(|(_, p)| p.full_name().chars().count())
                .max()
                .unwrap()
                + 2;
            let matching_vars: BTreeSet<String> = sets
                .iter()
                .flat_map(|(s, _)| s.token_states.keys().filter(|v| v.starts_with(subvar)))
                .map(|s| s.to_string())
                .collect();

            // Print table header.
            print!("{:>width$}", "", width = name_width);
            for matched_var in matching_vars.iter() {
                print!(
                    "{:>width$}",
                    matched_var,
                    width = max(matched_var.len() + 2, 7)
                );
            }
            println!();

            // Print each row of the table.
            for (set, p) in sets {
                print!("{:<width$}", p.full_name(), width = name_width);

                for matched_var in matching_vars.iter() {
                    let var_fmt_width = max(matched_var.len() + 2, 7);
                    if let Some(v) = set.token_states.get(matched_var.as_str()) {
                        match v {
                            TokenState::Enabled(_) => {
                                print!("{:>var$}", "SET".green(), var = var_fmt_width);
                            }
                            TokenState::Disabled(_) => {
                                print!("{:>var$}", "UNSET".red(), var = var_fmt_width);
                            }
                        }
                    } else if let Some(_span) = set.glob {
                        print!("{:>var$}", "UNSET".red(), var = var_fmt_width);
                    } else {
                        print!("{:>var$}", "", var = var_fmt_width);
                    }
                }

                println!();
            }
        } else {
            bail!("Please specify a token to track when blaming an incremental variable.")
        }
    } else {
        let vals = overlay_table.compute_variable(&profile, target_var)?;
        blame_format(&vals, config);
    }

    Ok(())
}

/// Helper function to print the detailed lineart for span metadata on variable contents.
fn blame_format(tokens: &[Span], config: &DrydockConfig) {
    let mut seen = HashMap::new();
    for t in tokens {
        let idx = seen.len();
        seen.entry(t.extra).or_insert(idx);
    }
    let mut f = source_span::fmt::Formatter::new();
    f.set_viewbox(None);
    f.hide_line_numbers();

    let metrics = source_span::DEFAULT_METRICS;
    let src_buf: source_span::SourceBuffer<(), _, _> = source_span::SourceBuffer::new(
        tokens.iter().flat_map(|t| t.fragment().chars().map(Ok)),
        Position::default(),
        metrics,
    );

    let total_len: usize = tokens.iter().map(|t| t.fragment().chars().count()).sum();
    let mut chars_seen: usize = 0;

    for t in tokens {
        let token_len = t.fragment().chars().count();
        let span = source_span::Span::new(
            Position::new(0, chars_seen),
            Position::new(0, chars_seen + token_len),
            Position::new(0, max(chars_seen + token_len + 1, total_len)),
        );
        chars_seen += token_len;
        f.add(span, Some(span_label(t, config)), Style::Help);
    }
    let display_span = source_span::Span::new(
        Position::new(0, 0),
        Position::new(0, chars_seen),
        Position::new(0, chars_seen + 1),
    );
    let formatted = f.render(src_buf.iter(), display_span, &metrics).unwrap();
    println!("{}", formatted);
}

/// Helper function to extract information from the metadata in a [Span] and generate a
/// a short label to print to a user for the source of that [Span].
/// The generated labels look like `my-overlay/profiles/some/profile:L50`, which would
/// correspond to line number 50 from the `some/profile` profile of the `my-overlay` overlay.
fn span_label(p: &Span, _config: &DrydockConfig) -> String {
    let profile_dir: OsString = OsString::from("profiles");
    let mut ancestors = p.extra.ancestors();

    while let Some(parent) = ancestors.next() {
        if let Some(ext) = parent.file_name() {
            if ext == profile_dir {
                let prefix = ancestors.nth(1).unwrap();
                let truncated_path = p.extra.strip_prefix(prefix).unwrap();
                return format!("{}:L{}", truncated_path.display(), p.location_line());
            }
        }
    }

    format!("{}:L{}", p.extra.display(), p.location_line())
}
