// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

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
    sequence::{delimited, preceded, separated_pair},
    IResult,
};

use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str, &'a Path>;
pub type ValueMap<'a> = HashMap<&'a str, RVal<'a>>;

/// An enum corresponding to the values that can be assigned to a variable. The two variants
/// correspond to either a literal string or an in-place variable expansion (e.g. "${FOO}").
/// A variable expansion can then recursively contain literal strings and more variable expansions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    /// A verbatim section of text, e.g. "foo".
    Literal(Span<'a>),
    /// A variable expansion site, e.g. `${MY_VAR}`.
    Expansion {
        /// The name of the variable being expanded, e.g. `MY_VAR` for `${MY_VAR}`.
        name: Span<'a>,
        // TODO(cjmcdonald): Make `value` just hold an `RVal`.
        /// The value of the variable at the time of the expansion.
        ///
        /// ## Example
        /// After the following snippet the expansion `${SPAM}` would have a `value` field with
        /// a single [Value::Literal] consisting of `"breakfast"`. The expansion `${HAM}` would
        /// have a `value` field of a single [Value::Expansion] corresponding to the variable
        /// `FOOBAR`, and that variable expansion's `value` field would be [None], as no value for
        /// the `FOOBAR` variable has been set yet.
        /// ```text
        /// SPAM="breakfast"
        /// HAM="${FOOBAR}"
        /// ```
        value: Option<Vec<Value<'a>>>,
    },
}

impl fmt::Display for Value<'_> {
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

/// An [RVal] is the complete expression on the right-hand side of a variable assignment, e.g.
/// `FOO="spam $HAM eggs"` would have an [RVal] of `"spam $HAM eggs"`. In this example, the
/// [RVal] would have the [Value]s of a Literal("spam "), an Expansion{name: "HAM"}, and another
/// Literal(" eggs").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RVal<'a> {
    pub vals: Vec<Value<'a>>,
}

static PLACEHOLDER_RVAL: RVal<'static> = RVal { vals: Vec::new() };

impl<'a> RVal<'a> {
    pub fn placeholder() -> &'static RVal<'static> {
        &PLACEHOLDER_RVAL
    }

    pub(super) fn new(vals: Vec<Value<'a>>) -> Self {
        Self { vals }
    }
}

impl fmt::Display for RVal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for val in self.vals.iter() {
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}

/// Parser entry point for the entirety of a `make.conf` file. Expects the full body of the file
/// as a single [Span] as input.
pub fn full_parse(mut input: Span<'_>) -> anyhow::Result<ValueMap<'_>> {
    let mut assignment_map: ValueMap = HashMap::new();

    // This parser loop re-assigns the remaining text to the `input` variable as fragments
    // are consumed by each sub-parser.
    while !input.is_empty() {
        if let Ok((new_input, _)) = comment_line(input) {
            input = new_input;
        } else if let Ok((new_input, (lval, rval))) = assignment(input, &assignment_map) {
            assignment_map.insert(lval.fragment(), rval);
            input = new_input;
        } else {
            // Consume any stray leading whitespace, or return an error if we cannot parse further.
            let (new, _) =
                multispace1::<Span, nom::error::VerboseError<Span>>(input).map_err(|_| {
                    anyhow!(
                        "Syntax error at line {line_number}:\n\n\
                        {full_line}\n\
                        {caret:>column$}\n\n\
                        Invalid fragment (expected a variable assignment or comment).
                    ",
                        line_number = input.location_line(),
                        full_line = std::str::from_utf8(input.get_line_beginning()).unwrap(),
                        caret = '^',
                        column = input.get_column(),
                    )
                })?;
            input = new;
        }
    }

    Ok(assignment_map)
}

/// Parser to recognize a commented line in a `make.conf` file.
pub fn comment_line(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    recognize(preceded(complete::char('#'), complete::not_line_ending))(input)
}

/// Parser to recognize a full assignment expression, e.g. `FOO="$BAR $BAZ"`.
fn assignment<'a>(
    input: Span<'a>,
    prior_asn: &HashMap<&str, RVal<'a>>,
) -> IResult<Span<'a>, (Span<'a>, RVal<'a>)> {
    let quoted_rval_parser = |i| quoted_rval(i, prior_asn);
    let unquoted_rval_parser = |i| unquoted_rval(i, prior_asn);

    separated_pair(
        variable,
        tag("="),
        alt((quoted_rval_parser, unquoted_rval_parser)),
    )(input)
}

/// Parser to recognize a properly quoted [RVal].
///
/// Spec reference:
/// https://dev.gentoo.org/~ulm/pms/head/pms.html#x1-470005.2.4
///
/// Line continuations are not currently handled properly.
fn quoted_rval<'a>(
    input: Span<'a>,
    prior_asgn: &HashMap<&str, RVal<'a>>,
) -> IResult<Span<'a>, RVal<'a>> {
    // Helper to grab the current definition for a variable and immediately inline
    // that definition into the Expansion's value field.
    let var_expansion_inliner = |v| {
        if let Value::Expansion { name, .. } = v {
            let value = prior_asgn.get(&*name).map(|x| x.vals.clone());
            Value::Expansion { name, value }
        } else {
            v
        }
    };

    map(
        delimited(
            tag("\""),
            many0(alt((literal, map(expansion, var_expansion_inliner)))),
            tag("\""),
        ),
        |vals| RVal { vals },
    )(input)
}

/// Parser to recognize unquoted rvalues, as much as possible.
///
/// These are violations of the PMS, but the ability to correctly parse these is needed to support
/// the few organic usages within the Chrome OS tree.
fn unquoted_rval<'a>(
    input: Span<'a>,
    prior_asgn: &HashMap<&str, RVal<'a>>,
) -> IResult<Span<'a>, RVal<'a>> {
    // Helper to grab the current definition for a variable and immediately inline
    // that definition into the Expansion's value field.
    let var_expansion_inliner = |v| {
        if let Value::Expansion { name, .. } = v {
            let value = prior_asgn.get(&*name).map(|x| x.vals.clone());
            Value::Expansion { name, value }
        } else {
            v
        }
    };

    let not_ws = |c: char| !c.is_ascii_whitespace();
    // Our best guess for an unquoted rvalue is everything up until the next piece of whitespace.
    let unquoted_literal = map(take_while1(not_ws), Value::Literal);

    map(
        preceded(
            multispace0,
            many0(map(
                alt((expansion, unquoted_literal)),
                var_expansion_inliner,
            )),
        ),
        RVal::new,
    )(input)
}

/// Parser to recognize string literals.
fn literal(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(is_not("$\""), Value::Literal)(input)
}

/// Parser to recognize variable names.
fn variable(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let leading_symbol = |c| is_alphabetic(c as u8);
    let trailing_symbol = |c| is_alphanumeric(c as u8) || c == '_';
    recognize(preceded(
        take_while1(leading_symbol),
        take_while(trailing_symbol),
    ))(input)
}

/// Parser to recognize variable expansions in rvalues.
fn expansion(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(
        preceded(
            tag("$"),
            alt((delimited(tag("{"), variable, tag("}")), variable)),
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
    fn null_span(text: &str) -> Span<'_> {
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

    const TWENTY_FIVE_LAUGHS_UNBRACED: &str = r#"
LOL="lol"
LOL="$LOL $LOL $LOL $LOL $LOL"
LOL="$LOL $LOL $LOL $LOL $LOL"
"#;

    #[test]
    fn test_25_laughs_unbraced_evaluation() {
        let res = full_parse(null_span(TWENTY_FIVE_LAUGHS_UNBRACED));
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
