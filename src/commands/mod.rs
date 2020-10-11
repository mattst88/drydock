use std::{cmp::max, collections::HashMap, fmt::Write, path::PathBuf};
use std::{collections::HashSet, str::FromStr};

use clap::ArgMatches;
use source_span::{fmt::Style, Position};

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

pub fn blame(config: &config::Config, sub_args: &ArgMatches) -> anyhow::Result<()> {
    let target = sub_args.value_of("profile").unwrap();
    let profile = ProfileKey::from_str(target)?;

    let target_var = sub_args.value_of("variable").unwrap();
    let overlay_table = build_overlay_map(&config)?;
    if is_incremental_variable(target_var) {
        unimplemented!();
    } else {
        let vals = overlay_table.compute_variable(&profile, target_var)?;
        blame_format(&vals, config);
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

fn blame_format(tokens: &[Span], config: &config::Config) {
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
        tokens
            .into_iter()
            .flat_map(|t| t.fragment().chars().map(|c| Ok(c))),
        Position::default(),
        metrics,
    );

    let total_len: usize = tokens
        .into_iter()
        .map(|t| t.fragment().chars().count())
        .sum();
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

fn span_label(p: &Span, config: &config::Config) -> String {
    for common_path in config.get_array("overlay_paths").unwrap() {
        let common_prefix: PathBuf = PathBuf::from(common_path.into_str().unwrap());
        if let Ok(new_path) = p.extra.strip_prefix(common_prefix) {
            return format!("{}:L{}", new_path.display(), p.location_line());
        }
    }

    format!("{}:L{}", p.extra.display(), p.location_line())
}
