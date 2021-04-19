//! Module for handling the behavior of Portage incremental variables.
//! Incremental variables differ from 'regular' variables and are more appropriately
//! thought of as unordered sets of string tokens that are either enabled or disabled.
//! PMS entry: https://dev.gentoo.org/~ulm/pms/head/pms.html#x1-560005.3.1

use std::collections::BTreeMap;

use nom::{
    bytes::complete::tag, bytes::complete::take_while1, character::complete::multispace0,
    sequence::preceded, IResult,
};

use super::profile_parser::Span;

/// Enum indicating whether a token is explicitly enabled or disabled in this context.
///
/// The interior [Span] is the original source location where this state was set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenState<'a> {
    Enabled(Span<'a>),
    Disabled(Span<'a>),
}

/// A set of [TokenState]s representing the cumulative value of an incremental variable in some
/// context.
///
/// An individual token has three possible states in a context: enabled, disabled, or not
/// mentioned at all. These three states are represented by the presence or absence of a
/// [TokenState] in the set.
#[derive(Debug, Default)]
pub struct TokenSet<'a> {
    pub glob: Option<Span<'a>>,
    pub token_states: BTreeMap<&'a str, TokenState<'a>>,
}

impl<'a> TokenSet<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parser to split a slice of [Span]s, which individually might contain multiple tokens, into a [TokenSet].
    pub fn from_raw_spans(raw_spans: &[Span<'a>]) -> anyhow::Result<Self> {
        let ws_enabled_token = preceded(multispace0, enabled_token);
        let ws_disabled_token = preceded(multispace0, disabled_token);
        let ws_reset_glob = preceded(multispace0, reset_glob);

        let mut glob: Option<Span> = None;
        let mut token_states: BTreeMap<&str, TokenState> = BTreeMap::new();

        'spans: for mut span in raw_spans.iter().cloned() {
            /*
            Each iteration of this loop consumes a single token along with any amount of leading
            whitespace and then updates `span` to point at the end of the consumed input.
            The loop breaks once `span` is empty or only consists of whitespace.

            Example:
            start:  span = " foo bar -* "
            loop 1: span = " bar -* "
            loop 2: span = " -* "
            loop 3: span = " "
            loop 4: breaks
            */
            loop {
                if let Ok((sp, val)) = ws_reset_glob(span) {
                    // Matching a literal "-*" (a 'reset glob') is equivalent to disabling every
                    // token. We model this by clearing all values in the map so far, storing the
                    // parsed span of the reset glob.
                    glob = Some(val);
                    span = sp;
                    token_states.clear();
                } else if let Ok((sp, val)) = ws_disabled_token(span) {
                    // Match a token to be disabled (e.g. "-foo").
                    span = sp;
                    token_states.insert(val.fragment(), TokenState::Disabled(val));
                } else if let Ok((sp, val)) = ws_enabled_token(span) {
                    // Match a token to be enabled (e.g. "foo").
                    span = sp;
                    token_states.insert(val.fragment(), TokenState::Enabled(val));
                } else if span.trim().is_empty() {
                    // If `span` is empty or only consists of whitespace, proceed to the next item
                    // in the outer loop.
                    continue 'spans;
                } else {
                    // TODO(cjmcdonald): Add typed error handling here.
                    anyhow::bail!("Unable to parse fragment: {}", span)
                }
            }
        }

        Ok(Self { glob, token_states })
    }

    /// Merge this [TokenSet] with another, with the values of `other` superceding the values
    /// of `self.
    pub fn merge(&mut self, other: Self) {
        let Self {
            glob,
            mut token_states,
        } = other;
        if let Some(glob_span) = glob {
            for val in self.token_states.values_mut() {
                *val = TokenState::Disabled(glob_span)
            }
        }
        self.token_states.append(&mut token_states);
    }
}

/// Transform a [TokenSet] into a Vec of [Span]s of the enabled tokens in this set.
/// This output would correspond to the actual contents of the variable as used
/// by Portage.
impl<'a> From<TokenSet<'a>> for Vec<Span<'a>> {
    fn from(set: TokenSet<'a>) -> Self {
        set.token_states
            .into_iter()
            .filter_map(|(_, v)| match v {
                TokenState::Disabled(_) => None,
                TokenState::Enabled(s) => Some(s),
            })
            .collect()
    }
}

/// Parser to recognize a single token. Tokens are just assumed to be not-whitespace.
fn token(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let not_ws = |c: char| !c.is_ascii_whitespace();
    take_while1(not_ws)(input)
}

/// Parser to recognize an enabled token.
fn enabled_token(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    token(input)
}

/// Parser to recognize a disabled token.
fn disabled_token(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    preceded(tag("-"), token)(input)
}

/// Parser to recognize the special glob which disables all tokens in the current context.
fn reset_glob(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    tag("-*")(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use lazy_static::lazy_static;
    use proptest::{prop_assert_eq, proptest};

    use std::path::Path;

    /// Helper function to create a [Span] of text with no associated file path.
    fn null_span(text: &str) -> Span<'_> {
        lazy_static! {
            static ref NULL_PATH: &'static Path = Path::new("");
        }
        Span::new_extra(text, &**NULL_PATH)
    }

    #[test]
    fn test_from_raw_spans_basic() {
        let token_set = TokenSet::from_raw_spans(&[null_span(" foo bar baz ")]).unwrap();
        assert!(matches!(
            token_set.token_states["foo"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["bar"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["baz"],
            TokenState::Enabled(_)
        ));
        assert!(token_set.glob.is_none());
    }

    #[test]
    fn test_from_raw_spans_basic_negation() {
        let token_set = TokenSet::from_raw_spans(&[null_span("foo bar baz -foo")]).unwrap();
        assert!(matches!(
            token_set.token_states["foo"],
            TokenState::Disabled(_)
        ));
        assert!(matches!(
            token_set.token_states["bar"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["baz"],
            TokenState::Enabled(_)
        ));
        assert!(token_set.glob.is_none());
    }

    #[test]
    fn test_from_raw_spans_multiple_spans() {
        let token_set =
            TokenSet::from_raw_spans(&[null_span("foo bar baz "), null_span("spam ham eggs")])
                .unwrap();
        assert!(matches!(
            token_set.token_states["foo"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["bar"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["baz"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["spam"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["ham"],
            TokenState::Enabled(_)
        ));
        assert!(matches!(
            token_set.token_states["eggs"],
            TokenState::Enabled(_)
        ));
        assert!(token_set.glob.is_none());
    }

    #[test]
    fn test_from_raw_spans_glob_basic() {
        let token_set = TokenSet::from_raw_spans(&[null_span("foo bar baz -*")]).unwrap();
        assert!(token_set.glob.is_some());
    }

    #[test]
    #[should_panic(expected = "no entry found for key")]
    fn test_from_raw_spans_glob_clears_existing() {
        let token_set = TokenSet::from_raw_spans(&[null_span("foo bar baz -*")]).unwrap();
        assert!(matches!(
            token_set.token_states["foo"],
            TokenState::Enabled(_)
        ));
    }

    #[test]
    #[should_panic(expected = "no entry found for key")]
    fn test_from_raw_spans_glob_clears_existing_multiple_spans() {
        let token_set = TokenSet::from_raw_spans(&[
            null_span("foo bar"),
            null_span("spam ham"),
            null_span("eggs -*"),
        ])
        .unwrap();
        assert!(matches!(
            token_set.token_states["foo"],
            TokenState::Enabled(_)
        ));
    }

    #[test]
    fn test_from_raw_spans_redefinition_after_glob() {
        let token_set = TokenSet::from_raw_spans(&[null_span("foo bar baz -* -foo")]).unwrap();
        assert!(token_set.glob.is_some());
        assert!(matches!(
            token_set.token_states["foo"],
            TokenState::Disabled(_)
        ));
    }

    proptest! {
        #[test]
        fn test_random_from_raw_span(s in r#"([ \t]*[A-Za-z]+)"#) {
            let spans = &[null_span(&s)];
            let token_set = TokenSet::from_raw_spans(spans).unwrap();
            let output_spans: Vec<Span<'_>> = token_set.into();
            let fragment = output_spans[0].fragment();
            prop_assert_eq!(&s.trim(), fragment);
        }

        /// Test that creating a single [TokenSet] from two spans produces the same output
        /// as creating two separate [TokenSet]s from those spans and merging them.
        #[test]
        fn test_merged_pair_same_as_single_parse(s1 in r#"([ \t]+-?[A-Za-z]+)*[ \t]*"#,
                                                 s2 in r#"([ \t]+-?[A-Za-z]+)*[ \t]*"#) {
            let both_spans = &[null_span(&s1), null_span(&s2)];
            let from_both = TokenSet::from_raw_spans(both_spans).unwrap();

            let mut only_first = TokenSet::from_raw_spans(&[null_span(&s1)]).unwrap();
            let only_second = TokenSet::from_raw_spans(&[null_span(&s2)]).unwrap();
            only_first.merge(only_second);

            let mut from_both_spans: Vec<Span<'_>> = from_both.into();
            from_both_spans.sort_by(|l, r| l.fragment().cmp(r.fragment()));

            let mut pairwise_spans: Vec<Span<'_>> = only_first.into();
            pairwise_spans.sort_by(|l, r| l.fragment().cmp(r.fragment()));

            prop_assert_eq!(from_both_spans, pairwise_spans);
        }
    }
}
