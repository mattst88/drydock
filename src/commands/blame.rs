// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::cmp::max;
use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::bail;
use clap::ArgMatches;
use colored::Colorize;
use source_span::{fmt::Style, Position};

use crate::{
    config::DrydockConfig,
    portage::{
        overlay::{build_overlay_map, OverlayTable},
        profile_parser::Span,
        variables::TokenState,
        ProfileKey,
    },
};

/// Print the parsed syntax tree for the contents of a variable with the annotated location
/// of each leaf in the tree.
///
/// Portage really has two types of variables: incremental and non-incremental. The `blame` report
/// for each type is fundamentally different and is handled separately.
///
/// ## Non-incremental variables
/// Non-incremental variables behave like familiar variables in other settings: the most
/// recent definition is the one that is used.
///
/// ## Incremental variables
/// Incremental variables function more like sets of tokens, with each node in the inheritance
/// tree having a separate definition of that variable and the final value being the concatenated
/// union of every profile's definition of that variable.
/// See [blame_incremental] for the implementation of incremental variable blame reporting.
pub fn blame(config: &DrydockConfig, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("profile").unwrap();
    let profile = ProfileKey::from_str(target)?;

    let mut target_values = sub_args.values_of("variable").unwrap();
    let target_var = target_values.next().unwrap();
    let overlay_table = build_overlay_map(config)?;
    if overlay_table.is_incremental_variable(&profile, target_var) {
        if let Some(subtoken_prefix) = target_values.next() {
            blame_incremental(&overlay_table, &profile, target_var, subtoken_prefix)?;
        } else {
            bail!("Please specify a token to track when blaming an incremental variable.");
        }
    } else {
        let vals = overlay_table.compute_variable(&profile, target_var)?;
        blame_format(&vals, config);
    }

    Ok(())
}

/// Print to stdout a formatted table of the inheritance hierarchy for `profile` and the effect
/// each parent in that hierarchy has on the matched subtokens of the incremental variable
/// `variable`.
///
/// The `subtoken_prefix` argument matches any token which it is a literal prefix of, e.g. an
/// argument of "foo" would match against the tokens "foo", "foobar", and "foolish".
fn blame_incremental(
    table: &OverlayTable,
    profile: &ProfileKey,
    variable: &str,
    subtoken_prefix: &str,
) -> anyhow::Result<()> {
    let sets = table.compute_incremental_variable(&profile, variable)?;
    let greatest_profile_name_length = sets
        .iter()
        // PMS 3.1.5: Profile names only have characters in the set [A-Za-z0-9_-] so counting
        // characters to calculate the displayed string width is valid here.
        .map(|(_, p)| p.full_name().chars().count())
        .max()
        .unwrap();
    let matching_vars: BTreeSet<String> = sets
        .iter()
        .flat_map(|(s, _)| {
            s.token_states
                .keys()
                .filter(|v| v.starts_with(subtoken_prefix))
        })
        .map(|s| s.to_string())
        .collect();

    write_blame_table_header(
        &mut std::io::stdout(),
        greatest_profile_name_length,
        matching_vars.iter().map(|s| s.as_str()),
    )?;

    // Print each row of the table.
    for (set, profile) in sets {
        let row_entries: Vec<(&str, BlameStatus)> = matching_vars
            .iter()
            .map(|s| s.as_ref())
            .map(|var| match set.token_states.get(var) {
                Some(TokenState::Enabled(_)) => (var, BlameStatus::Enabled),
                Some(TokenState::Disabled(_)) => (var, BlameStatus::Disabled),
                None => {
                    if set.glob.is_some() {
                        (var, BlameStatus::Disabled)
                    } else {
                        (var, BlameStatus::Missing)
                    }
                }
            })
            .collect();

        write_blame_table_row(
            &mut std::io::stdout(),
            greatest_profile_name_length,
            profile.full_name(),
            row_entries.into_iter(),
        )?;
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
    let profile_dir = PathBuf::from("profiles");
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

/// The minimum column size in the blame table is the width of the word 'unset' plus a space on
/// each side.
const MIN_COLUMN_SIZE: usize = 7;

/// Write the formatted first line of a blame table to the provided writer.
fn write_blame_table_header<'a>(
    writer: &mut impl Write,
    max_profile_name_len: usize,
    column_titles: impl Iterator<Item = &'a str>,
) -> anyhow::Result<()> {
    // Add two spaces between the profile names and table entries for enhanced visual clarity.
    let initial_padding = max_profile_name_len + 2;
    write!(writer, "{:>width$}", "", width = initial_padding)?;
    for column in column_titles {
        write!(
            writer,
            "{:>width$}",
            column,
            // Pad the width of the column name by 2 to allow for a space on each side.
            width = max(column.len() + 2, MIN_COLUMN_SIZE)
        )?;
    }
    writeln!(writer)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum BlameStatus {
    Enabled,
    Disabled,
    Missing,
}

/// Write a single formatted row of the blame table to the provided writer.
fn write_blame_table_row<'a>(
    writer: &mut impl Write,
    max_profile_name_len: usize,
    row_name: &str,
    row_entries: impl Iterator<Item = (&'a str, BlameStatus)>,
) -> anyhow::Result<()> {
    // Pad two spaces between the profile names and table entries for enhanced visual clarity.
    let initial_padding = max_profile_name_len + 2;

    write!(writer, "{:<width$}", row_name, width = initial_padding)?;

    for (token, status) in row_entries {
        // Pad two spaces between each token name to visually separate each column.
        let var_fmt_width = max(token.len() + 2, MIN_COLUMN_SIZE);

        match status {
            BlameStatus::Enabled => {
                write!(writer, "{:>var$}", "SET".green(), var = var_fmt_width)?;
            }
            BlameStatus::Disabled => {
                write!(writer, "{:>var$}", "UNSET".red(), var = var_fmt_width)?;
            }
            BlameStatus::Missing => {
                write!(writer, "{:>var$}", "", var = var_fmt_width)?;
            }
        }
    }

    writeln!(writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_write_blame_table_header_basic() -> anyhow::Result<()> {
        let columns = ["foo", "bar", "baz"];
        let padding = 5usize;

        let mut buf = Vec::<u8>::new();
        write_blame_table_header(&mut buf, padding, columns.iter().cloned())?;

        let output = String::from_utf8(buf)?;
        assert_eq!(output, "           foo    bar    baz\n");
        Ok(())
    }

    #[test]
    fn test_write_blame_table_row_basic() -> anyhow::Result<()> {
        let row_entries = vec![
            ("foo", BlameStatus::Missing),
            ("bar", BlameStatus::Disabled),
            ("baz", BlameStatus::Enabled),
        ];
        let padding = 5usize;

        let mut buf = Vec::<u8>::new();
        write_blame_table_row(&mut buf, padding, "foo", row_entries.into_iter())?;

        let output = String::from_utf8(buf)?;

        assert_eq!(
            output,
            format!("foo    {:>7}{:>7}{:>7}\n", "", "UNSET".red(), "SET".green())
        );

        Ok(())
    }

    #[test]
    fn test_span_label_basic() {
        let path = Path::new("/usr/src/overlay/profiles/foo/bar");
        let test_span = Span::new_extra("123456789", path);
        let config = Default::default();
        let output = span_label(&test_span, &config);
        assert_eq!(output, "overlay/profiles/foo/bar:L1");
    }
}
