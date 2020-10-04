use std::path::PathBuf;

use anyhow;
use lazy_static::lazy_static;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{self, multispace0, multispace1, one_of, satisfy},
    character::{is_alphabetic, is_alphanumeric},
    combinator::{map, recognize},
    multi::{self, many0, many1},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};

use regex;

use crate::portage::profile_parser as parse;

lazy_static! {
    static ref PARENT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^\s*(?:(?P<repo>[^:\s]+):)?(?P<path>[A-Za-z0-9_\-/.]+)").unwrap();
}

lazy_static! {
    static ref LAYOUT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^repo-name\s=\s([A-Za-z0-9_-]+)").unwrap();
}

pub fn parse_parent_file(body: &str) -> anyhow::Result<Vec<(Option<String>, PathBuf)>> {
    many1(preceded(
        many0(preceded(multispace0, parse::comment_line)), // comment or blank line
        alt((
            map(
                preceded(
                    multispace0,
                    separated_pair(is_not(": \n"), tag(":"), is_not(" \n")),
                ), // absolute path
                |(r, p)| (Some(String::from(r)), PathBuf::from(p)),
            ),
            map(preceded(multispace0, is_not(" \n")), |v| {
                (None, PathBuf::from(v))
            }),
        )),
    ))(body)
    .finish()
    .map(|(_, v): (&str, Vec<(Option<String>, PathBuf)>)| v)
    .map_err(|_| anyhow::anyhow!("parser's busted :("))
}

/// Parse layout.conf and return only the overlay's name for now.
/// repo-name = chromiumos
pub fn parse_layout_conf(body: &str) -> anyhow::Result<&str> {
    LAYOUT_REGEX
        .captures(body)
        .map(|m| m.get(1))
        .flatten()
        .map(|m| m.as_str())
        .ok_or(anyhow::anyhow!("problem parsing layout.conf"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            parse_parent_file(SAMPLE).unwrap(),
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
            parse_parent_file(NO_LINEFEED).unwrap(),
            vec![(
                Some("chromiumos".to_owned()),
                PathBuf::from("features/selinux")
            ),]
        )
    }

    #[test]
    fn test_tiny_no_linefeed_parent_file_parse() {
        assert_eq!(
            parse_parent_file("..").unwrap(),
            vec![(None, PathBuf::from("..")),]
        )
    }

    #[test]
    fn test_layout_regex() {
        assert_eq!(parse_layout_conf(LAYOUT_SAMPLE).unwrap(), "chromiumos")
    }
}
