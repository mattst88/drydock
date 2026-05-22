// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::path::PathBuf;

use anyhow::anyhow;
use lazy_static::lazy_static;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::multispace0,
    combinator::{map, value},
    multi::many0,
    sequence::{preceded, separated_pair},
    Parser,
};

use crate::portage::profile::ProfileReference;
use crate::portage::profile_parser::{self as parse, Span};

lazy_static! {
    static ref LAYOUT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^repo-name\s=\s([A-Za-z0-9_-]+)").unwrap();
}

/// Parser for a Portage profile 'parents' file. These files are a newline-delimited list
/// of either relative profile paths (e.g. "../..") within the same repository or absolute
/// profile paths with a leading repository name, e.g. "gentoo:default/linux/amd64/23.0"
/// Note that the leading name is *not* a path: it is the `repo-name` variable
/// declared in the repository's layout.conf
pub fn parse_parent_file(body: Span) -> anyhow::Result<Vec<ProfileReference>> {
    let comment_line = preceded(multispace0, parse::comment_line);
    let comment_parser = value(None, comment_line);

    let absolute_reference = map(
        preceded(
            multispace0,
            separated_pair(is_not(": \n"), tag(":"), is_not(" \n")),
        ), // absolute path
        |(repo, path): (Span, Span)| {
            Some(ProfileReference::Absolute {
                repo_name: String::from(*repo),
                path: PathBuf::from(*path),
            })
        },
    );

    let relative_reference = map(preceded(multispace0, is_not(" \n:")), |path: Span| {
        Some(ProfileReference::Relative {
            path: PathBuf::from(*path),
        })
    });

    many0(alt((
        comment_parser,
        absolute_reference,
        relative_reference,
    ))).parse(body)
    .map(|(_, v): (Span, Vec<Option<_>>)| v.into_iter().flatten().collect())
    .map_err(|e| match e {
        nom::Err::Error(e) => {
            anyhow!("Parse error at line {}: {:?}", e.input.location_line(), e.code)
        }
        nom::Err::Failure(e) => {
            anyhow!("Parse failure at line {}: {:?}", e.input.location_line(), e.code)
        }
        nom::Err::Incomplete(_) => {
            anyhow!("Ambiguous parser failure")
        }
    })
}

/// Parse layout.conf and return the repository name (`repo-name` field).
/// repo-name = gentoo
pub fn parse_layout_conf(body: Span<'_>) -> anyhow::Result<&str> {
    LAYOUT_REGEX
        .captures(body.fragment())
        .map(|m| m.get(1))
        .flatten()
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow::anyhow!("problem parsing layout.conf"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    fn null_span(text: &'static str) -> Span<'static> {
        lazy_static! {
            static ref NULL_PATH: &'static Path = Path::new("");
        }
        Span::new_extra(text, *NULL_PATH)
    }
    const SAMPLE: &str = "# This is a comment.
..
../../base
gentoo:default/linux/amd64/23.0

";

    const LAYOUT_SAMPLE: &str = "
cache-format = md5-dict
masters = gentoo
profile-formats = portage-2
repo-name = my-repo
thin-manifests = true
use-manifests = strict
";

    #[test]
    fn test_parent_file_parse() {
        assert_eq!(
            parse_parent_file(null_span(SAMPLE)).unwrap(),
            vec![
                ProfileReference::Relative { path: "..".into() },
                ProfileReference::Relative {
                    path: "../../base".into()
                },
                ProfileReference::Absolute {
                    repo_name: "gentoo".into(),
                    path: "default/linux/amd64/23.0".into()
                },
            ]
        );
    }

    const NO_LINEFEED: &str = "gentoo:default/linux/amd64/23.0";

    #[test]
    fn test_no_linefeed_parent_file_parse() {
        assert_eq!(
            parse_parent_file(null_span(NO_LINEFEED)).unwrap(),
            vec![ProfileReference::Absolute {
                repo_name: "gentoo".into(),
                path: "default/linux/amd64/23.0".into()
            },]
        )
    }

    #[test]
    fn test_tiny_no_linefeed_parent_file_parse() {
        assert_eq!(
            parse_parent_file(null_span("..")).unwrap(),
            vec![ProfileReference::Relative { path: "..".into() },]
        )
    }

    #[test]
    fn test_layout_regex() {
        assert_eq!(
            parse_layout_conf(null_span(LAYOUT_SAMPLE)).unwrap(),
            "my-repo"
        )
    }
}
