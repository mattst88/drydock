#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{self, multispace0, multispace1, one_of, satisfy},
    character::{is_alphabetic, is_alphanumeric},
    combinator::{map, recognize},
    multi::{self, many0},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

static INCREMENTAL_VARIABLES: &[&str] = &[
    "USE",
    "USE_EXPAND",
    "USE_EXPAND_HIDDEN",
    "CONFIG_PROTECT",
    "CONFIG_PROTECT_MASK",
    "IUSE_IMPLICIT",
    "USE_EXPAND_IMPLICIT",
    "USE_EXPAND_UNPREFIXED",
    "ENV_UNSET",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value<'a> {
    Literal(&'a str),
    Expansion {
        name: &'a str,
        value: Option<Vec<Arc<Value<'a>>>>,
    },
}

impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Literal(s) => write!(f, "{}", s),
            Value::Expansion { name, value } => {
                if let Some(values) = value {
                    for val in values {
                        write!(f, "{}", val)?;
                    }
                    Ok(())
                } else {
                    write!(f, "!!${{{}}}!!", name)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Assignment<'a> {
    lval: &'a str,
    rval: Vec<Arc<Value<'a>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RVal<'a> {
    vals: Vec<Arc<Value<'a>>>,
}

impl<'a> fmt::Display for RVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for val in self.vals.iter() {
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}

fn full_parse(mut input: &str) -> IResult<&str, HashMap<&str, RVal>> {
    let mut assignment_map: HashMap<&str, RVal> = HashMap::new();

    while input != "" {
        if let Ok((new_input, _)) = comment_line(input) {
            input = new_input;
        } else if let Ok((new_input, (lval, rval))) = assignment(input, &assignment_map) {
            assignment_map.insert(lval, rval);
            input = new_input;
        } else {
            let (new, _) = multispace1(input)?;
            input = new;
        }
    }

    Ok((input, assignment_map))
}

fn comment_line(input: &str) -> IResult<&str, &str> {
    recognize(preceded(complete::char('#'), complete::not_line_ending))(input)
}

fn multi_line(input: &str) -> IResult<&str, Vec<&str>> {
    multi::many1(preceded(complete::multispace0, comment_line))(input)
}

fn assignment<'a, 'b>(
    input: &'a str,
    prior_asn: &'b HashMap<&'a str, RVal<'a>>,
) -> IResult<&'a str, (&'a str, RVal<'a>)> {
    let quoted_rval_parser = |i| quoted_rval(i, prior_asn);
    preceded(
        multispace0,
        tuple((
            terminated(variable, preceded(multispace0, tag("="))),
            preceded(multispace0, quoted_rval_parser),
        )),
    )(input)
}

fn quoted_rval<'a, 'b>(
    input: &'a str,
    prior_asgn: &'b HashMap<&'a str, RVal<'a>>,
) -> IResult<&'a str, RVal<'a>> {
    map(
        terminated(
            preceded(
                tag("\""),
                many0(map(alt((literal, expansion)), |v| match v {
                    v @ Value::Literal { .. } => Arc::new(v),
                    Value::Expansion { name, .. } => {
                        let value = prior_asgn.get(name).map(|a| a.vals.clone());
                        Arc::new(Value::Expansion { name, value })
                    }
                })),
            ),
            tag("\""),
        ),
        |vals| RVal { vals },
    )(input)
}

fn literal(input: &str) -> IResult<&str, Value> {
    map(is_not("$\""), |s| Value::Literal(s))(input)
}

fn variable(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        satisfy(|c| c.is_ascii_alphabetic()),
        many0(satisfy(|c| c.is_ascii_alphanumeric() || c == '_')),
    ))(input)
}

fn expansion(input: &str) -> IResult<&str, Value> {
    map(
        preceded(
            tag("$"),
            alt((
                map(tuple((tag("{"), variable, tag("}"))), |res| res.1),
                variable,
            )),
        ),
        |s| Value::Expansion {
            name: s,
            value: None,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_SAMPLE: &str = r#"# Copyright (c) 2015 The Chromium OS Authors. All rights reserved.
# Distributed under the terms of the GNU General Public License v2

# Settings that are common to all host sdks.  Do not place any board specific
# settings in here, or settings for cross-compiled targets.
#
# See "man 5 make.conf" and "man 5 portage" for the available options.

# Dummy setting so we can use the same append form below.
USE=""

# Various global settings.
USE="${USE} hardened multilib pic pie -introspection -cracklib"

# Custom USE flag ebuilds can use to determine whether it's going into the sdk
# or into a target board.
USE="${USE} cros_host"

# Disable all x11 USE flags for packages within chroot.
USE="${USE} -gtk2 -gtk3 -qt4"

# Enable extended attributes support in our sdk tools.
USE="${USE} xattr"
# But disable using them in the sdk itself for now.
USE="${USE} -filecaps"

# No need to track power in the sdk.
USE="${USE} -power_management"

# We don't boot things inside the sdk.
USE="${USE} -openrc"

# Disable vala inside the sdk
USE="${USE} -vala"

# We only have one rootfs.
USE="${USE} -split-usr"

# Various runtime features that control emerge behavior.
# See "man 5 make.conf" for details.
FEATURES="allow-missing-manifests buildpkg clean-logs -collision-protect
            -ebuild-locks force-mirror -merge-sync -pid-sandbox
            parallel-install -preserve-libs sandbox -strict userfetch
            userpriv usersandbox -unknown-features-warn network-sandbox"

# This is used by profiles/base/profile.bashrc to figure out that we
# are targeting the cros-sdk (in all its various modes).  It should
# be utilized nowhere else!
CROS_SDK_HOST="cros-sdk-host"

# Qemu targets we care about.
QEMU_SOFTMMU_TARGETS="aarch64 arm i386 mips mips64 mips64el mipsel x86_64"
QEMU_USER_TARGETS="aarch64 arm i386 mips mips64 mips64el mipsel x86_64"

# Various compiler defaults.  Should be no arch-specific bits here.
CFLAGS="-O2 -pipe"
LDFLAGS="-Wl,-O2 -Wl,--as-needed"

# We want to migrate away from this at some point.
SYMLINK_LIB="yes"

# Default target(s) for python-r1.eclass
PYTHON_TARGETS="-python2_7 python3_6"
PYTHON_SINGLE_TARGET="-python2_7 python3_6"

# Use clang as the default compiler.
CC="x86_64-pc-linux-gnu-clang"
CXX="x86_64-pc-linux-gnu-clang++"
LD="x86_64-pc-linux-gnu-ld.lld"


    "#;

    #[test]
    fn test_comment_parse() {
        let res = comment_line(FULL_SAMPLE);
        let (_, capture) = res.unwrap();

        assert_eq!(
            capture,
            "# Copyright (c) 2015 The Chromium OS Authors. All rights reserved."
        );
    }

    #[test]
    fn test_multi_comment_parse() {
        let res = multi_line(FULL_SAMPLE);
        let (_, capture) = res.unwrap();
        assert_eq!(
            capture,
            vec![
                "# Copyright (c) 2015 The Chromium OS Authors. All rights reserved.",
                "# Distributed under the terms of the GNU General Public License v2",
                "# Settings that are common to all host sdks.  Do not place any board specific",
                "# settings in here, or settings for cross-compiled targets.",
                "#",
                r#"# See "man 5 make.conf" and "man 5 portage" for the available options."#,
                "# Dummy setting so we can use the same append form below."
            ]
        );
    }

    const ASSIGN: &str = r#"USE="${USE} hardened multilib pic pie -introspection -cracklib""#;
    #[test]
    fn test_single_assignment_parse() {
        let res = assignment(ASSIGN, &HashMap::new());
        let (out, asgn) = res.unwrap();

        assert_eq!(
            (
                "USE",
                RVal {
                    vals: vec![
                        Arc::new(Value::Expansion {
                            name: "USE",
                            value: None
                        }),
                        Arc::new(Value::Literal(
                            " hardened multilib pic pie -introspection -cracklib"
                        ))
                    ]
                }
            ),
            asgn
        );
        assert_eq!(out, "");
    }

    const MULTI_ASSIGN: &str = r#"
USE="foo"
USE="${USE} bar"
"#;

    #[test]
    fn test_multi_assignment_parse() {
        let res = full_parse(MULTI_ASSIGN);
        let (out, res) = res.unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "USE",
            RVal {
                vals: vec![
                    Arc::new(Value::Expansion {
                        name: "USE",
                        value: Some(vec![Arc::new(Value::Literal("foo"))]),
                    }),
                    Arc::new(Value::Literal(" bar")),
                ],
            },
        );
        assert_eq!(res, expected);

        assert_eq!(out, "");
    }

    #[test]
    fn test_multi_assign_evaluation() {
        let res = full_parse(MULTI_ASSIGN);
        let (out, res) = res.unwrap();
        assert_eq!(out, "");
        assert_eq!("foo bar", format!("{}", res["USE"]));
    }

    #[test]
    fn test_full_example_parse() {
        let res = full_parse(FULL_SAMPLE);
        let (out, res) = res.unwrap();
        assert_eq!(out, "");
        assert_eq!(res.len(), 13);
    }
}
