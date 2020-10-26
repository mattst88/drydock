use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{
    cmp::{Eq, PartialEq},
    str::FromStr,
};

use anyhow::bail;
use nom_locate::LocatedSpan;

use crate::parse;
use crate::portage::profile_parser::{full_parse, RVal, Value};

use super::profile_parser::Span;

const PARENT_FILE: &str = "parent";
const MAKE_DEFAULTS: &str = "make.defaults";

rental! {
    mod rentals {
        use super::*;

        #[rental(debug, covariant)]
        pub struct FileMap {
            path: PathBuf,
            raw: String,
            span: LocatedSpan<&'raw str, &'path Path>,
        }

        #[rental(debug, covariant)]
        pub struct ParsedFile {
            file_map: Box<FileMap>,
            map: HashMap<&'file_map str, RVal<'file_map>>,
        }
    }
}

#[derive(Debug)]
pub struct Profile {
    pub name: String,
    pub parents: Vec<ProfileKey>,
    full_path: PathBuf,
    pub conf: Option<rentals::ParsedFile>,
}

impl Profile {
    pub fn new<T: Into<String>>(name: T, full_path: PathBuf) -> Self {
        Self {
            name: name.into(),
            parents: Default::default(),
            full_path,
            conf: Default::default(),
        }
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&RVal> {
        let conf: &rentals::ParsedFile = self.conf.as_ref().unwrap();
        conf.suffix().get(key.as_ref())
    }

    pub fn parse_parents(profile_path: &Path) -> anyhow::Result<Vec<(Option<String>, PathBuf)>> {
        let file_path = profile_path.join(PARENT_FILE);
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(&file_path)?;
        parse::parse_parent_file(Span::new_extra(&contents, file_path.as_path()))
    }

    pub fn parse_conf(&mut self) -> anyhow::Result<()> {
        match self.conf {
            Some(_) => Ok(()),
            None => {
                let conf_path = self.full_path.join(MAKE_DEFAULTS);
                let contents = if conf_path.is_file() {
                    fs::read_to_string(&conf_path)?
                } else {
                    String::new()
                };

                match rentals::FileMap::try_new(
                    conf_path,
                    |_| Ok(contents),
                    |s, p| {
                        let span: anyhow::Result<_> = Ok(LocatedSpan::new_extra(s, p));
                        span
                    },
                ) {
                    Ok(file_map) => {
                        match rentals::ParsedFile::try_new(Box::new(file_map), |filemap| {
                            full_parse(*filemap.suffix())
                        }) {
                            Ok(rentref) => {
                                self.conf = Some(rentref);
                                Ok(())
                            }
                            Err(e) => panic!(e),
                        }
                    }
                    Err(e) => panic!(e),
                }
            }
        }
    }
}

impl Hash for Profile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.parents.hash(state);
        self.full_path.hash(state);
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.parents == other.parents
            && self.full_path == other.full_path
    }
}
impl Eq for Profile {}

/// A type representing the unambiguous name & location of a profile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileKey {
    data: String,
}

impl FromStr for ProfileKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split(':');
        if let (Some(overlay), Some(name)) = (splits.next(), splits.next()) {
            Ok(Self::new(overlay, name))
        } else {
            bail! {"Unable to parse profile key from string. A profile key must be of the form overlay:path/to/profile."}
        }
    }
}

impl ProfileKey {
    pub fn new<T: Into<String>, U: Into<String>>(overlay: T, name: U) -> Self {
        Self {
            data: format!("{}:{}", overlay.into(), name.into()),
        }
    }

    pub fn overlay(&self) -> &str {
        self.data.split(':').next().unwrap()
    }

    pub fn profile(&self) -> &str {
        self.data.split(':').nth(1).unwrap()
    }

    pub fn full_name(&self) -> &str {
        self.data.as_str()
    }
}

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

    pub fn feed(&mut self, rval: &RVal<'a>, profile: &'a ProfileKey) -> MuncherState<'a> {
        for val in rval.vals.clone().into_iter().rev() {
            self.exploration_stack.push((val, profile));
        }

        self.munch()
    }

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

pub enum MuncherState<'a> {
    Need((Span<'a>, &'a ProfileKey)),
    Done(Vec<Span<'a>>),
}

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

pub fn is_builtin_incremental_variable(variable: &str) -> bool {
    INCREMENTAL_VARIABLES.contains(&variable)
}
