use std::path::PathBuf;

use anyhow;
use lazy_static::lazy_static;
use regex;

lazy_static! {
    static ref PARENT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^\s*(?:(?P<repo>[^:\s]+):)?(?P<path>[A-Za-z0-9_\-/.]+)").unwrap();
}

lazy_static! {
    static ref LAYOUT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^repo-name\s=\s([A-Za-z0-9_-]+)").unwrap();
}

pub fn parse_parent_file(body: &str) -> Vec<(Option<String>, PathBuf)> {
    let mut output = Vec::new();
    for cap in PARENT_REGEX.captures_iter(body) {
        output.push((
            cap.get(1).map(|m| m.as_str().to_owned()),
            PathBuf::from(&cap["path"]),
        ))
    }
    output
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

    const SAMPLE: &str = "..
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
    fn test_parent_regex() {
        assert_eq!(
            parse_parent_file(SAMPLE),
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

    #[test]
    fn test_layout_regex() {
        assert_eq!(parse_layout_conf(LAYOUT_SAMPLE).unwrap(), "chromiumos")
    }
}
