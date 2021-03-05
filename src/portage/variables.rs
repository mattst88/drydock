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
pub struct TokenSet<'a> {
    pub glob: Option<Span<'a>>,
    pub token_states: BTreeMap<&'a str, TokenState<'a>>,
}

impl Default for TokenSet<'_> {
    fn default() -> Self {
        TokenSet {
            glob: Default::default(),
            token_states: Default::default(),
        }
    }
}

impl<'a> TokenSet<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parser to split a slice of [Span]s, which individually might contain multiple tokens, into a [TokenSet].
    pub fn from_raw_spans(raw_spans: &[Span<'a>]) -> Self {
        let ws_enabled_token = preceded(multispace0, enabled_token);
        let ws_disabled_token = preceded(multispace0, disabled_token);
        let ws_reset_glob = preceded(multispace0, reset_glob);

        let mut glob: Option<Span> = None;
        let mut token_states: BTreeMap<&str, TokenState> = BTreeMap::new();

        'span: for mut span in raw_spans.iter().cloned() {
            loop {
                if let Ok((s, val)) = ws_reset_glob(span) {
                    glob = Some(val);
                    span = s;
                    token_states.clear();
                } else if let Ok((s, val)) = ws_disabled_token(span) {
                    span = s;
                    token_states.insert(val.fragment(), TokenState::Disabled(val));
                } else if let Ok((s, val)) = ws_enabled_token(span) {
                    span = s;
                    token_states.insert(val.fragment(), TokenState::Enabled(val));
                } else {
                    continue 'span;
                }
            }
        }

        Self { glob, token_states }
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

    /// Transform a [TokenSet] into a Vec of [Span]s of the enabled tokens in this set.
    /// This output would correspond to the actual contents of the variable as used by Portage.
    pub fn into_spans(self) -> Vec<Span<'a>> {
        self.token_states
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
