use std::collections::BTreeMap;

use nom::{
    bytes::complete::tag, bytes::complete::take_while1, character::complete::multispace0,
    sequence::preceded, IResult,
};

use super::profile_parser::Span;

pub enum TokenState<'a, 'b> {
    Enabled(Span<'a, 'b>),
    Disabled(Span<'a, 'b>),
}

pub struct TokenSet<'a, 'b> {
    pub glob: Option<Span<'a, 'b>>,
    pub token_states: BTreeMap<&'a str, TokenState<'a, 'b>>,
}

impl<'a, 'b> Default for TokenSet<'a, 'b> {
    fn default() -> Self {
        TokenSet {
            glob: Default::default(),
            token_states: Default::default(),
        }
    }
}

impl<'a, 'b> TokenSet<'a, 'b> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_raw_spans<'c>(raw_spans: &'c [Span<'a, 'b>]) -> Self {
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

    pub fn to_spans(self) -> Vec<Span<'a, 'b>> {
        self.token_states
            .into_iter()
            .filter_map(|(_, v)| match v {
                TokenState::Disabled(_) => None,
                TokenState::Enabled(s) => Some(s),
            })
            .collect()
    }
}

fn token<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Span<'a, 'b>> {
    let not_ws = |c: char| !c.is_ascii_whitespace();
    take_while1(not_ws)(input)
}

fn enabled_token<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Span<'a, 'b>> {
    token(input)
}

fn disabled_token<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Span<'a, 'b>> {
    preceded(tag("-"), token)(input)
}

fn reset_glob<'a, 'b>(input: Span<'a, 'b>) -> IResult<Span<'a, 'b>, Span<'a, 'b>> {
    tag("-*")(input)
}
