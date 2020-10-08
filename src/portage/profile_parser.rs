use std::fmt;
use std::{collections::HashMap, path::Path};

use anyhow::anyhow;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while, take_while1},
    character::complete::{self, multispace0, multispace1},
    character::is_alphabetic,
    character::is_alphanumeric,
    combinator::{map, recognize},
    multi::many0,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use nom_locate::LocatedSpan;

pub type Span<'data, 'path> = LocatedSpan<&'data str, &'path Path>;
pub type ValueMap<'data, 'path> = HashMap<&'data str, RVal<'data, 'path>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a, 'b> {
    Literal(Span<'a, 'b>),
    Expansion {
        name: Span<'a, 'b>,
        value: Option<Vec<Value<'a, 'b>>>,
    },
}

impl<'a, 'b> fmt::Display for Value<'a, 'b> {
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
pub struct RVal<'a, 'b> {
    pub vals: Vec<Value<'a, 'b>>,
}

static PLACEHOLDER_RVAL: RVal<'static, 'static> = RVal { vals: Vec::new() };

impl<'a, 'b> RVal<'a, 'b> {
    pub fn placeholder() -> &'static RVal<'static, 'static> {
        &PLACEHOLDER_RVAL
    }

    fn new(vals: Vec<Value<'a, 'b>>) -> Self {
        Self { vals }
    }
}

impl<'a, 'b> fmt::Display for RVal<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for val in self.vals.iter() {
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}

pub fn full_parse<'a, 'b>(mut input: Span<'a, 'b>) -> anyhow::Result<ValueMap<'a, 'b>> {
    let mut assignment_map: ValueMap = HashMap::new();

    while input.fragment() != &"" {
        if let Ok((new_input, _)) = comment_line(input) {
            input = new_input;
        } else if let Ok((new_input, (lval, rval))) = assignment(input, &assignment_map) {
            assignment_map.insert(lval.fragment(), rval);
            input = new_input;
        } else {
            let (new, _) = multispace1::<Span, nom::error::VerboseError<Span>>(input)
                .map_err(|e| anyhow!(e.to_string()))?;
            input = new;
        }
    }

    Ok(assignment_map)
}

pub fn comment_line<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Span<'a, 'b>> {
    recognize(preceded(complete::char('#'), complete::not_line_ending))(input)
}

fn assignment<'a: 'c, 'b, 'c>(
    input: Span<'a, 'b>,
    prior_asn: &'c HashMap<&'a str, RVal<'a, 'b>>,
) -> IResult<Span<'a, 'b>, (Span<'a, 'b>, RVal<'a, 'b>)> {
    let quoted_rval_parser = |i| quoted_rval(i, prior_asn);
    let unquoted_rval_parser = |i| unquoted_rval(i, prior_asn);
    preceded(
        multispace0,
        tuple((
            terminated(variable, preceded(multispace0, tag("="))),
            alt((
                preceded(multispace0, quoted_rval_parser),
                unquoted_rval_parser,
            )),
        )),
    )(input)
}

fn quoted_rval<'a: 'c, 'b, 'c>(
    input: Span<'a, 'b>,
    prior_asgn: &'c HashMap<&'a str, RVal<'a, 'b>>,
) -> IResult<Span<'a, 'b>, RVal<'a, 'b>> {
    map(
        terminated(
            preceded(
                tag("\""),
                many0(map(alt((literal, expansion)), |v| match v {
                    v @ Value::Literal { .. } => v,
                    Value::Expansion { name, .. } => {
                        let value = prior_asgn.get(name.fragment()).map(|a| a.vals.clone());
                        Value::Expansion { name, value }
                    }
                })),
            ),
            tag("\""),
        ),
        |vals| RVal { vals },
    )(input)
}

fn unquoted_rval<'a: 'c, 'b, 'c>(
    input: Span<'a, 'b>,
    prior_asgn: &'c HashMap<&'a str, RVal<'a, 'b>>,
) -> IResult<Span<'a, 'b>, RVal<'a, 'b>> {
    map(
        preceded(
            multispace0,
            many0(map(
                alt((expansion, map(is_not(" \t\n"), Value::Literal), expansion)),
                |v| match v {
                    v @ Value::Literal { .. } => v,
                    Value::Expansion { name, .. } => {
                        let value = prior_asgn.get(name.fragment()).map(|a| a.vals.clone());
                        Value::Expansion { name, value }
                    }
                },
            )),
        ),
        RVal::new,
    )(input)
}

fn literal<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Value<'a, 'b>> {
    map(is_not("$\""), Value::Literal)(input)
}

fn variable<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Span<'a, 'b>> {
    let leading_symbol = |c| is_alphabetic(c as u8);
    let trailing_symbol = |c| is_alphanumeric(c as u8) || c == '_';
    recognize(preceded(
        take_while1(leading_symbol),
        take_while(trailing_symbol),
    ))(input)
}

fn expansion<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Value<'a, 'b>> {
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
    use lazy_static::lazy_static;
    use nom::Slice;
    fn null_span(text: &'static str) -> Span<'static, 'static> {
        lazy_static! {
            static ref NULL_PATH: &'static Path = Path::new("");
        }
        Span::new_extra(text, *NULL_PATH)
    }
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
        let res = comment_line(null_span(FULL_SAMPLE));
        let (_, capture) = res.unwrap();

        assert_eq!(
            *capture.fragment(),
            "# Copyright (c) 2015 The Chromium OS Authors. All rights reserved."
        );
    }

    const ASSIGN: &str = r#"USE="${USE} hardened multilib pic pie -introspection -cracklib""#;
    #[test]
    fn test_single_assignment_parse() {
        let assign_span = null_span(ASSIGN);
        let res = assignment(null_span(ASSIGN), &HashMap::new());
        let (out, asgn) = res.unwrap();

        assert_eq!(
            (
                assign_span.slice(0..3),
                RVal {
                    vals: vec![
                        Value::Expansion {
                            name: assign_span.slice(7..10),
                            value: None
                        },
                        Value::Literal(assign_span.slice(11..62))
                    ]
                }
            ),
            asgn
        );
        assert_eq!(out, assign_span.slice(63..));
    }

    const MULTI_ASSIGN: &str = r#"
USE="foo"
USE="${USE} bar"
"#;

    #[test]
    fn test_multi_assignment_parse() {
        let multi_assign_span = null_span(MULTI_ASSIGN);
        let res = full_parse(multi_assign_span);
        let res = res.unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            "USE",
            RVal {
                vals: vec![
                    Value::Expansion {
                        name: multi_assign_span.slice(18usize..21usize),
                        value: Some(vec![Value::Literal(multi_assign_span.slice(6..9))]),
                    },
                    Value::Literal(multi_assign_span.slice(22..26)),
                ],
            },
        );
        assert_eq!(res, expected);
    }

    #[test]
    fn test_multi_assign_evaluation() {
        let res = full_parse(null_span(MULTI_ASSIGN));
        let res = res.unwrap();
        assert_eq!("foo bar", format!("{}", res["USE"]));
    }

    const MANY_ASSIGN: &str = r#"
USE="foo"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
USE="${USE} bar"
"#;

    #[test]
    fn test_many_assign_evaluation() {
        let res = full_parse(null_span(MANY_ASSIGN));
        let res = res.unwrap();
        assert_eq!(
            "foo bar bar bar bar bar bar bar bar bar",
            format!("{}", res["USE"])
        );
    }

    const TWENTY_FIVE_LAUGHS: &str = r#"
LOL="lol"
LOL="${LOL} ${LOL} ${LOL} ${LOL} ${LOL}"
LOL="${LOL} ${LOL} ${LOL} ${LOL} ${LOL}"
"#;

    const TWENTY_FIVE_LAUGHS_EXPANDED: &str = "lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol lol";

    #[test]
    fn test_25_laughs_evaluation() {
        let res = full_parse(null_span(TWENTY_FIVE_LAUGHS));
        let res = res.unwrap();
        assert_eq!(format!("{}", res["LOL"]), TWENTY_FIVE_LAUGHS_EXPANDED);
    }

    #[test]
    fn test_full_example_parse() {
        let res = full_parse(null_span(FULL_SAMPLE));
        let res = res.unwrap();
        assert_eq!(res.len(), 13);
    }

    const BAD_QUOTES_FULL_SAMPLE: &str = r#"# Copyright (c) 2015 The Chromium OS Authors. All rights reserved.
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
CC=x86_64-pc-linux-gnu-clang
CXX=x86_64-pc-linux-gnu-clang++
LD=x86_64-pc-linux-gnu-ld.lld


    "#;

    #[test]
    fn test_bad_quotes_full_example_parse() {
        let res = full_parse(null_span(BAD_QUOTES_FULL_SAMPLE));
        let res = res.unwrap();
        assert_eq!(res.len(), 13);
    }

    #[test]
    fn test_bad_quotes_full_example_eval_unquoted() {
        let res = full_parse(null_span(BAD_QUOTES_FULL_SAMPLE));
        let res = res.unwrap();
        assert_eq!(res["CC"].to_string(), "x86_64-pc-linux-gnu-clang");
    }

    #[test]
    fn test_bad_quotes_full_example_eval_quoted() {
        let res = full_parse(null_span(BAD_QUOTES_FULL_SAMPLE));
        let res = res.unwrap();
        assert_eq!(res["PYTHON_TARGETS"].to_string(), "-python2_7 python3_6");
    }
}
