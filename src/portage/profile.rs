// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{
    cmp::{Eq, PartialEq},
    str::FromStr,
};

use anyhow::bail;
use nom_locate::LocatedSpan;
use self_cell::self_cell;

use crate::parse;
use crate::portage::profile_parser::{full_parse, RVal, Value, ValueMap};

use super::profile_parser::Span;

const PARENT_FILE: &str = "parent";
const MAKE_DEFAULTS: &str = "make.defaults";

/// Portage variables that are defined by the Package Manager Spec to always be treated as
/// incremental.
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

/// Helper function to determine if a variable is a Portage built-in incremental variable.
pub fn is_builtin_incremental_variable(variable: &str) -> bool {
    INCREMENTAL_VARIABLES.contains(&variable)
}

/// Owned storage for a profile configuration file.
#[derive(Debug)]
struct FileMapOwner {
    path: PathBuf,
    raw: String,
}

type FileSpan<'a> = LocatedSpan<&'a str, &'a Path>;

self_cell!(
    /// Owns the file path and raw content, with a dependent span borrowing from both.
    struct FileMap {
        owner: FileMapOwner,
        #[covariant]
        dependent: FileSpan,
    }
    impl { Debug }
);

self_cell!(
    /// Owns a [FileMap] and a dependent variable map borrowing string data from it.
    pub struct ParsedFile {
        owner: Box<FileMap>,
        #[covariant]
        dependent: ValueMap,
    }
    impl { Debug }
);

/// A single instance of a Portage profile.
#[derive(Debug)]
pub struct Profile {
    /// Profile name as declared in layout.conf
    pub name: String,
    /// [ProfileKey]s of the profiles declared as parents of this profile.
    pub parents: Vec<ProfileKey>,
    /// Full filesystem path to this profile.
    full_path: PathBuf,
    /// Parsed configuration file contents, if they have been evaluated yet.
    pub conf: Option<ParsedFile>,
}

impl Profile {
    /// Create a new instance from a profile name and a full filesystem path.
    pub fn new<T: Into<String>>(name: T, full_path: PathBuf) -> Self {
        // TODO: validate if the name and full path are coherent.
        Self {
            name: name.into(),
            parents: Default::default(),
            full_path,
            conf: Default::default(),
        }
    }

    /// Look up a variable definition in this profile's [ParsedFile].
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&RVal<'_>> {
        self.conf.as_ref().unwrap().borrow_dependent().get(key.as_ref())
    }

    /// Parse the `parents` file of a profile, with a non-existent file signifying no parents.
    pub fn parse_parents(profile_path: &Path) -> anyhow::Result<Vec<ProfileReference>> {
        let file_path = profile_path.join(PARENT_FILE);
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(&file_path)?;
        parse::parse_parent_file(Span::new_extra(&contents, file_path.as_path()))
    }

    /// Load a `make.conf` or `make.defaults` file in this profile if it exists and parse it.
    pub fn parse_and_ingest_conf(&mut self) -> anyhow::Result<()> {
        if self.conf.is_some() {
            return Ok(());
        }

        let conf_path = self.full_path.join(MAKE_DEFAULTS);
        let raw = if conf_path.is_file() {
            fs::read_to_string(&conf_path)?
        } else {
            String::new()
        };

        let file_map = FileMap::new(
            FileMapOwner { path: conf_path, raw },
            |owner| LocatedSpan::new_extra(owner.raw.as_str(), owner.path.as_path()),
        );

        let parsed_file = ParsedFile::try_new(Box::new(file_map), |fm| {
            full_parse(*fm.borrow_dependent())
        })?;

        self.conf = Some(parsed_file);
        Ok(())
    }
}

impl Hash for Profile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Note: The `conf` field is omitted from the hash calculation.
        self.name.hash(state);
        self.parents.hash(state);
        self.full_path.hash(state);
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        // Note: The `conf` field is omitted from the equality calculation.
        self.name == other.name
            && self.parents == other.parents
            && self.full_path == other.full_path
    }
}
impl Eq for Profile {}

/// The unambiguous name & location of a profile.
///
/// Looks like "repo-name:path/to/profile".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileKey {
    data: String,
}

impl FromStr for ProfileKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_by_colon = s.split(':');
        if let (Some(repo_name), Some(name)) = (split_by_colon.next(), split_by_colon.next()) {
            if !repo_name.is_empty() && !name.is_empty() {
                return Ok(Self::new(repo_name, name));
            }
        }
        bail! {"Unable to parse profile key from string: {}\
        \nA profile key must be of the form repo-name:path/to/profile.", s}
    }
}

impl ProfileKey {
    pub fn new<T: Into<String>, U: Into<String>>(repo_name: T, name: U) -> Self {
        Self {
            data: format!("{}:{}", repo_name.into(), name.into()),
        }
    }

    pub fn repo_name(&self) -> &str {
        self.data.split(':').next().unwrap()
    }

    pub fn profile(&self) -> &str {
        self.data.split(':').nth(1).unwrap()
    }

    pub fn full_name(&self) -> &str {
        self.data.as_str()
    }
}

/// A potentially ambigious reference to another profile.
///
/// Parent relationships between profiles can either be specified in an absolute fashion, e.g.
/// `gentoo:default/linux/amd64/23.0`, or as a relative path to the parent file e.g. `../..`.
/// A relative reference cannot be unambiguously made into a [ProfileKey] without knowing the
/// profile in which it was declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileReference {
    Absolute { repo_name: String, path: PathBuf },
    Relative { path: PathBuf },
}

/// A state machine to turn parsed syntax trees into flattened variables.
///
/// Traverses a syntax tree with two stacks: an 'output' stack of processed values and an
/// 'exploration' stack of values to recursively explore, expand, and push back onto the stack
/// as needed.
///
/// A variable definition may contain references to other variables which might be defined
/// in an entirely different profile, so while evaluating the definition of a variable we
/// may need to indicate that a yet-undefined variable has been found. This isn't an error
/// but we don't have enough information to proceed, so we return a [MuncherState] to the
/// caller to indicate that they need to supply a definition for a given variable in a given
/// context.
pub struct ValueMuncher<'a> {
    output_tokens: Vec<Span<'a>>,
    exploration_stack: Vec<(Value<'a>, &'a ProfileKey)>,
}

impl<'a> ValueMuncher<'a> {
    pub fn new() -> Self {
        Self {
            output_tokens: Default::default(),
            exploration_stack: Default::default(),
        }
    }

    /// Push a [RVal] onto the exploration stack of this [ValueMuncher].
    ///
    /// Upon receiving [MuncherState::Need], the caller is expected to provide the needed
    /// variable definition by calling [`ValueMuncher::feed()`] again
    pub fn feed(&mut self, rval: &RVal<'a>, profile: &'a ProfileKey) -> MuncherState<'a> {
        for val in rval.vals.clone().into_iter().rev() {
            self.exploration_stack.push((val, profile));
        }

        self.munch()
    }

    /// Process the exploration stack as much as possible. Returns a [MuncherState] enum indicating
    /// either that all input was processed or that a variable definition is required from the
    /// caller.
    fn munch(&mut self) -> MuncherState<'a> {
        loop {
            match self.exploration_stack.pop() {
                None => return MuncherState::Done(std::mem::take(&mut self.output_tokens)),
                Some((Value::Literal(a), _)) => self.output_tokens.push(a),
                Some((Value::Expansion { name, value }, p)) => {
                    if let Some(vals) = value {
                        for value in vals.into_iter().rev() {
                            self.exploration_stack.push((value, p));
                        }
                    } else {
                        return MuncherState::Need((name, p));
                    }
                }
            }
        }
    }
}

/// Status enum indicating the processing state of a [ValueMuncher]. See the documentation on
/// [ValueMuncher] for more explanation of these states.
pub enum MuncherState<'a> {
    Need((Span<'a>, &'a ProfileKey)),
    Done(Vec<Span<'a>>),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::test_data_dir;

    use proptest::prelude::*;

    fn null_span(text: &str) -> Span<'_> {
        Span::new_extra(text, &Path::new(""))
    }

    #[test]
    fn test_profilekey_parse_basic() -> anyhow::Result<()> {
        let key = ProfileKey::from_str("foo:path/to/profile")?;
        assert_eq!(key.profile(), "path/to/profile");
        assert_eq!(key.repo_name(), "foo");
        Ok(())
    }

    #[test]
    fn test_profilekey_parse_bad_lead() {
        assert!(ProfileKey::from_str(":path/to/profile").is_err());
    }

    #[test]
    fn test_profilekey_parse_bad_end() {
        assert!(ProfileKey::from_str("foo:").is_err());
    }

    #[test]
    fn test_profilekey_parse_relative() {
        assert!(ProfileKey::from_str("../..").is_err());
    }

    #[test]
    fn test_parent_parse_basic() -> anyhow::Result<()> {
        let test_profile_path = test_data_dir(&[
            "test-tree",
            "spam",
            "profiles",
            "special_feature",
            "extra_special_feature",
        ]);

        let parents = Profile::parse_parents(&test_profile_path)?;
        assert_eq!(
            parents,
            vec![
                ProfileReference::Relative { path: "..".into() },
                ProfileReference::Absolute {
                    repo_name: "ham".into(),
                    path: "other".into()
                },
                ProfileReference::Absolute {
                    repo_name: "eggs".into(),
                    path: "base".into()
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn test_profile_ingest_basic() -> anyhow::Result<()> {
        let test_profile_path =
            test_data_dir(&["test-tree", "ham", "profiles", "base"]);

        let mut profile = Profile::new("ham", test_profile_path.clone());
        profile.parse_and_ingest_conf()?;

        assert_eq!(format!("{}", profile.get("BREAKFAST_FOOD").unwrap()), "ham");
        Ok(())
    }

    /// Assert that the [RVal] corresponding to the parse tree of `val_tree` in the following
    /// snippet is flattened to the value `"hamhamhamham"`:
    /// ```text
    /// ham="ham"
    /// spam="${ham}${ham}"
    /// spam2="${ham}${ham}"
    /// val_tree="${spam}${spam2}"
    /// ```
    #[test]
    fn test_valuemuncher_assert_simple_tree_is_flattened() {
        let val_tree = RVal::new(vec![
            Value::Expansion {
                name: null_span("spam"),
                value: Some(vec![
                    Value::Literal(null_span("ham")),
                    Value::Literal(null_span("ham")),
                ]),
            },
            Value::Expansion {
                name: null_span("spam2"),
                value: Some(vec![
                    Value::Literal(null_span("ham")),
                    Value::Literal(null_span("ham")),
                ]),
            },
        ]);
        let key = ProfileKey::from_str("test:base").unwrap();
        let mut muncher = ValueMuncher::new();
        match muncher.feed(&val_tree, &key) {
            MuncherState::Need(_) => panic!("Should never return Need."),
            MuncherState::Done(output_vals) => {
                assert_eq!(output_vals, vec![null_span("ham"); 4])
            }
        }

        assert!(muncher.output_tokens.is_empty());
        assert!(muncher.exploration_stack.is_empty());
    }

    #[test]
    fn test_valuemuncher_assert_valuemuncher_is_safe_to_reuse() {
        let val_tree = RVal::new(vec![
            Value::Expansion {
                name: null_span("spam"),
                value: Some(vec![
                    Value::Literal(null_span("ham")),
                    Value::Literal(null_span("ham")),
                ]),
            },
            Value::Expansion {
                name: null_span("spam2"),
                value: Some(vec![
                    Value::Literal(null_span("ham")),
                    Value::Literal(null_span("ham")),
                ]),
            },
        ]);
        let key = ProfileKey::from_str("test:base").unwrap();
        let mut muncher = ValueMuncher::new();

        match muncher.feed(&val_tree, &key) {
            MuncherState::Need(_) => panic!("Should never return Need."),
            MuncherState::Done(output_vals) => {
                assert_eq!(output_vals, vec![null_span("ham"); 4])
            }
        }

        assert!(muncher.output_tokens.is_empty());
        assert!(muncher.exploration_stack.is_empty());

        // Feed the same Muncher twice, after it returns Done.
        match muncher.feed(&val_tree, &key) {
            MuncherState::Need(_) => panic!("Should never return Need."),
            MuncherState::Done(output_vals) => {
                assert_eq!(output_vals, vec![null_span("ham"); 4])
            }
        }

        assert!(muncher.output_tokens.is_empty());
        assert!(muncher.exploration_stack.is_empty());
    }

    proptest! {
        #[test]
        fn test_valuemuncher_assert_output_of_random_flat_literals_is_identical(
            vals in prop::collection::vec(prop::string::string_regex("[A-Za-z]+").unwrap(), 1..10)
        ) {
            let span_vals: Vec<Span> = vals.iter().map(|s| null_span(s.as_str())).collect();
            let literals = span_vals.iter().map(|s| Value::Literal(*s)).collect();
            let rval = RVal::new(literals);

            let placeholder_key = ProfileKey::from_str("test:base").unwrap();
            let mut muncher = ValueMuncher::new();
            match muncher.feed(&rval, &placeholder_key) {
                MuncherState::Need(_) => panic!("A list of all literals should never return Need."),
                MuncherState::Done(output_literals) => {
                    proptest::prop_assert_eq!(output_literals, span_vals);
                }
            };

            assert!(muncher.output_tokens.is_empty());
            assert!(muncher.exploration_stack.is_empty());
        }
    }
}
