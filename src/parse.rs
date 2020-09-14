use std::path::PathBuf;

use lazy_static::lazy_static;
use regex;

lazy_static! {
    static ref PARENT_REGEX: regex::Regex =
        regex::Regex::new(r"(?m)^\s*(?:(?P<repo>[^:\s]+):)?(?P<path>\S+)").unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "..
../../../../../targets/sdk
chromiumos:features/llvm

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
}
