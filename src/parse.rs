use std::path::PathBuf;

use lazy_static::lazy_static;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::multispace0,
    combinator::map,
    multi::{many0, many1},
    sequence::{preceded, separated_pair},
};

use crate::portage::profile_parser::{self as parse, Span};

lazy_static! {
    static ref PARENT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^\s*(?:(?P<repo>[^:\s]+):)?(?P<path>[A-Za-z0-9_\-/.]+)").unwrap();
}

lazy_static! {
    static ref LAYOUT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^repo-name\s=\s([A-Za-z0-9_-]+)").unwrap();
}

pub fn parse_parent_file(body: Span) -> anyhow::Result<Vec<(Option<String>, PathBuf)>> {
    many1(preceded(
        many0(preceded(multispace0, parse::comment_line)), // comment or blank line
        alt((
            map(
                preceded(
                    multispace0,
                    separated_pair(is_not(": \n"), tag(":"), is_not(" \n")),
                ), // absolute path
                |(r, p): (Span, Span)| {
                    (
                        Some(String::from(*r.fragment())),
                        PathBuf::from(p.fragment()),
                    )
                },
            ),
            map(preceded(multispace0, is_not(" \n")), |v: Span| {
                (None, PathBuf::from(v.fragment()))
            }),
        )),
    ))(body)
    .map(|(_, v): (Span, Vec<(Option<String>, PathBuf)>)| v)
    .map_err(|_| anyhow::anyhow!("parser's busted :("))
}

/// Parse layout.conf and return only the overlay's name for now.
/// repo-name = chromiumos
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
../../../../../targets/sdk
chromiumos:features/llvm

";

    const LAYOUT_SAMPLE: &str = "
cache-format = md5-dict
masters = portage-stable eclass-overlay
profile-formats = portage-2
repo-name = chromiumos
thin-manifests = true
use-manifests = strict
";

    #[test]
    fn test_parent_file_parse() {
        assert_eq!(
            parse_parent_file(null_span(SAMPLE)).unwrap(),
            vec![
                (None, PathBuf::from("..")),
                (None, PathBuf::from("../../../../../targets/sdk")),
                (
                    Some("chromiumos".to_owned()),
                    PathBuf::from("features/llvm")
                )
            ]
        );
    }

    const NO_LINEFEED: &str = "chromiumos:features/selinux";

    #[test]
    fn test_no_linefeed_parent_file_parse() {
        assert_eq!(
            parse_parent_file(null_span(NO_LINEFEED)).unwrap(),
            vec![(
                Some("chromiumos".to_owned()),
                PathBuf::from("features/selinux")
            ),]
        )
    }

    #[test]
    fn test_tiny_no_linefeed_parent_file_parse() {
        assert_eq!(
            parse_parent_file(null_span("..")).unwrap(),
            vec![(None, PathBuf::from("..")),]
        )
    }

    #[test]
    fn test_layout_regex() {
        assert_eq!(
            parse_layout_conf(null_span(LAYOUT_SAMPLE)).unwrap(),
            "chromiumos"
        )
    }
}
