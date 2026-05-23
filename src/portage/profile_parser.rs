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
    combinator::{map, recognize},
    multi::many0,
    sequence::{delimited, preceded, separated_pair},
    AsChar, IResult, Parser,
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
                multispace1::<Span, nom::error::Error<Span>>(input).map_err(|_| {
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
    recognize(preceded(complete::char('#'), complete::not_line_ending)).parse(input)
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
        alt((quoted_rval_parser, single_quoted_rval, unquoted_rval_parser)),
    ).parse(input)
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
            many0(alt((escaped_char, literal, map(expansion, var_expansion_inliner)))),
            tag("\""),
        ),
        |vals| RVal { vals },
    ).parse(input)
}

/// Parser to recognize single-quoted rvalues.
///
/// Single-quoted strings are fully literal — no variable expansion or escape sequences.
fn single_quoted_rval(input: Span<'_>) -> IResult<Span<'_>, RVal<'_>> {
    map(
        delimited(tag("'"), take_while(|c| c != '\''), tag("'")),
        |s: Span<'_>| {
            if s.is_empty() {
                RVal::new(vec![])
            } else {
                RVal::new(vec![Value::Literal(s)])
            }
        },
    ).parse(input)
}

/// Parser to recognize unquoted rvalues, as much as possible.
///
/// These are violations of the PMS, but the ability to correctly parse these is needed to support
/// these are violations of the PMS occasionally found in the wild.
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
    ).parse(input)
}

/// Parser to recognize string literals.
fn literal(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(is_not("$\"\\"), Value::Literal).parse(input)
}

/// Parser to recognize a backslash escape sequence (e.g. `\"` or `\\`).
///
/// The backslash and following character are both included in the returned span.
fn escaped_char(input: Span<'_>) -> IResult<Span<'_>, Value<'_>> {
    map(
        recognize(preceded(tag("\\"), complete::anychar)),
        Value::Literal,
    ).parse(input)
}

/// Parser to recognize variable names.
fn variable(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let leading_symbol = |c: char| c.is_alpha();
    let trailing_symbol = |c: char| c.is_alphanum() || c == '_';
    recognize(preceded(
        take_while1(leading_symbol),
        take_while(trailing_symbol),
    )).parse(input)
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
    ).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    fn null_span(text: &str) -> Span<'_> {
        lazy_static! {
            static ref NULL_PATH: &'static Path = Path::new("");
        }
        Span::new_extra(text, *NULL_PATH)
    }
    const FULL_SAMPLE: &str = r#"# Copyright 1999-2024 Gentoo Authors
# Distributed under the terms of the GNU General Public License v2

ARCH="amd64"
ACCEPT_KEYWORDS="amd64 ~amd64"

CHOST="x86_64-pc-linux-gnu"

# Compiler defaults.
CFLAGS="-O2 -pipe"
CXXFLAGS="${CFLAGS}"
FFLAGS="${CFLAGS}"
FCFLAGS="${CFLAGS}"

# Runtime features.
FEATURES="candy fixlafiles news parallel-fetch preserve-libs
            sandbox sfperms strict unknown-features-warn userpriv
            usersandbox usersync"

ABI="amd64"
DEFAULT_ABI="amd64"
MULTILIB_ABIS="amd64 x86"

PYTHON_TARGETS="python3_11 python3_12"
PYTHON_SINGLE_TARGET="python3_11"


    "#;

    #[test]
    fn test_comment_parse() {
        let res = comment_line(null_span(FULL_SAMPLE));
        let (_, capture) = res.unwrap();

        assert_eq!(
            *capture.fragment(),
            "# Copyright 1999-2024 Gentoo Authors"
        );
    }

    const ASSIGN: &str = r#"USE="${USE} hardened multilib pic pie -introspection -cracklib""#;
    #[test]
    fn test_single_assignment_parse() {
        let (out, (lval, rval)) = assignment(null_span(ASSIGN), &HashMap::new()).unwrap();
        assert_eq!(*lval.fragment(), "USE");
        assert_eq!(rval.vals.len(), 2);
        let Value::Expansion { name, value: None } = &rval.vals[0] else {
            panic!("expected expansion")
        };
        assert_eq!(*name.fragment(), "USE");
        let Value::Literal(lit) = &rval.vals[1] else {
            panic!("expected literal")
        };
        assert_eq!(*lit.fragment(), " hardened multilib pic pie -introspection -cracklib");
        assert_eq!(*out.fragment(), "");
    }

    const MULTI_ASSIGN: &str = r#"
USE="foo"
USE="${USE} bar"
"#;

    #[test]
    fn test_multi_assignment_parse() {
        let res = full_parse(null_span(MULTI_ASSIGN)).unwrap();
        let rval = &res["USE"];
        assert_eq!(rval.vals.len(), 2);
        let Value::Expansion { name, value: Some(inner) } = &rval.vals[0] else {
            panic!("expected expansion with inlined value")
        };
        assert_eq!(*name.fragment(), "USE");
        assert_eq!(inner.len(), 1);
        let Value::Literal(prev) = &inner[0] else {
            panic!("expected literal in inlined value")
        };
        assert_eq!(*prev.fragment(), "foo");
        let Value::Literal(appended) = &rval.vals[1] else {
            panic!("expected literal")
        };
        assert_eq!(*appended.fragment(), " bar");
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

    const BAD_QUOTES_FULL_SAMPLE: &str = r#"# Copyright 1999-2024 Gentoo Authors
# Distributed under the terms of the GNU General Public License v2

ARCH="amd64"
ACCEPT_KEYWORDS="amd64 ~amd64"

CHOST="x86_64-pc-linux-gnu"

# Compiler defaults.
CFLAGS="-O2 -pipe"
CXXFLAGS="${CFLAGS}"
FFLAGS="${CFLAGS}"
FCFLAGS="${CFLAGS}"

# Runtime features.
FEATURES="candy fixlafiles news parallel-fetch preserve-libs
            sandbox sfperms strict unknown-features-warn userpriv
            usersandbox usersync"

ABI="amd64"
DEFAULT_ABI="amd64"
MULTILIB_ABIS="amd64 x86"

PYTHON_TARGETS="python3_11 python3_12"

# Unquoted assignment (spec violation present in some repositories).
CC=x86_64-pc-linux-gnu-gcc


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
        assert_eq!(res["CC"].to_string(), "x86_64-pc-linux-gnu-gcc");
    }

    #[test]
    fn test_bad_quotes_full_example_eval_quoted() {
        let res = full_parse(null_span(BAD_QUOTES_FULL_SAMPLE));
        let res = res.unwrap();
        assert_eq!(res["PYTHON_TARGETS"].to_string(), "python3_11 python3_12");
    }

    const ESCAPED_QUOTES_SAMPLE: &str = r#"
BASE="foo"
OPTS="${BASE} --exclude \"bar\" --include \"baz\""
"#;

    #[test]
    fn test_escaped_quotes_parse() {
        let res = full_parse(null_span(ESCAPED_QUOTES_SAMPLE)).unwrap();
        assert_eq!(res["OPTS"].to_string(), r#"foo --exclude \"bar\" --include \"baz\""#);
    }
}
