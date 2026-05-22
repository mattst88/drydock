// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Integration tests for CLI usage of `drydock`.

mod test_util;

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile;

use test_util::test_data_dir;

fn drydock_with_test_tree() -> assert_cmd::Command {
    let mut cmd = Command::cargo_bin("drydock").unwrap();
    cmd.arg("--src-path").arg(test_data_dir(&["test-tree"]));
    cmd.into()
}

// ── test-tree tests (always run) ────────────────────────────────────────────

#[test]
fn eval_non_incremental_variable() -> anyhow::Result<()> {
    drydock_with_test_tree()
        .args(["eval", "-p", "ham:base", "BREAKFAST_FOOD"])
        .assert()
        .success()
        .stdout("ham\n");
    Ok(())
}

#[test]
fn eval_incremental_variable() -> anyhow::Result<()> {
    drydock_with_test_tree()
        .args(["eval", "-p", "spam:special_feature/extra_special_feature", "USE"])
        .assert()
        .success()
        .stdout("my_flag my_other_flag other_feature\n");
    Ok(())
}

#[test]
fn eval_inherited_variable() -> anyhow::Result<()> {
    // BREAKFAST_FOOD is set in ham:base; spam:special_feature overrides it to "spam".
    drydock_with_test_tree()
        .args(["eval", "-p", "spam:special_feature", "BREAKFAST_FOOD"])
        .assert()
        .success()
        .stdout("spam\n");
    Ok(())
}

#[test]
fn eval_incremental_variable_from_parent() -> anyhow::Result<()> {
    // eggs:base sets USE; spam:special_feature inherits and modifies it.
    let output = drydock_with_test_tree()
        .args(["eval", "-p", "eggs:base", "USE"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    assert!(stdout.contains("my_flag"));
    Ok(())
}

#[test]
fn parents_prints_tree() -> anyhow::Result<()> {
    drydock_with_test_tree()
        .args(["parents", "spam:special_feature/extra_special_feature"])
        .assert()
        .success()
        .stdout(
            "spam:special_feature/extra_special_feature\n\
            \tspam:special_feature\n\
            \t\tham:base\n\
            \t\t\teggs:base\n\
            \tham:other\n\
            \teggs:base\n",
        );
    Ok(())
}

#[test]
fn blame_incremental_variable_shows_set_status() -> anyhow::Result<()> {
    let output = drydock_with_test_tree()
        .args(["blame", "-p", "spam:special_feature/extra_special_feature", "USE:my_flag"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output)?;
    // eggs:base sets my_flag; it appears twice in the arborescence.
    assert!(output.contains("eggs:base"));
    assert!(output.contains("SET"));
    Ok(())
}

#[test]
fn blame_non_incremental_variable() -> anyhow::Result<()> {
    let output = drydock_with_test_tree()
        .args(["blame", "-p", "ham:base", "BREAKFAST_FOOD"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output)?;
    assert!(output.contains("ham"));
    Ok(())
}

#[test]
fn blame_incremental_missing_token_gives_helpful_error() -> anyhow::Result<()> {
    let output = drydock_with_test_tree()
        .args(["blame", "-p", "spam:special_feature", "USE"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output)?;
    assert!(stderr.contains("incremental variable"));
    assert!(stderr.contains("USE:<token>"));
    Ok(())
}

#[test]
fn config_generate_and_use() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let src_path = test_data_dir(&["test-tree"]);

    Command::cargo_bin("drydock")?
        .args(["config", "--default", "--config-file"])
        .arg(&config_path)
        .arg("--src-path")
        .arg(&src_path)
        .assert()
        .success();

    Command::cargo_bin("drydock")?
        .arg("--config-file")
        .arg(&config_path)
        .args(["eval", "-p", "ham:base", "BREAKFAST_FOOD"])
        .assert()
        .success()
        .stdout("ham\n");

    Ok(())
}

// ── Gentoo repo tests (skipped if /var/db/repos/gentoo absent) ──────────────

fn gentoo_repo_available() -> bool {
    Path::new("/var/db/repos/gentoo/profiles").is_dir()
}

macro_rules! gentoo_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() -> anyhow::Result<()> {
            if !gentoo_repo_available() {
                eprintln!("skipping: /var/db/repos/gentoo not available");
                return Ok(());
            }
            $body
        }
    };
}

fn drydock_gentoo() -> assert_cmd::Command {
    let mut cmd = Command::cargo_bin("drydock").unwrap();
    cmd.arg("--src-path").arg("/var/db/repos");
    cmd.into()
}

gentoo_test!(gentoo_eval_amd64_chost, {
    drydock_gentoo()
        .args(["eval", "-p", "gentoo:default/linux/amd64/23.0", "CHOST"])
        .assert()
        .success()
        .stdout("x86_64-pc-linux-gnu\n");
    Ok(())
});

gentoo_test!(gentoo_eval_alpha_chost, {
    drydock_gentoo()
        .args(["eval", "-p", "gentoo:default/linux/alpha/23.0", "CHOST"])
        .assert()
        .success()
        .stdout("alpha-unknown-linux-gnu\n");
    Ok(())
});

gentoo_test!(gentoo_eval_arm64_chost, {
    drydock_gentoo()
        .args(["eval", "-p", "gentoo:default/linux/arm64/23.0", "CHOST"])
        .assert()
        .success()
        .stdout("aarch64-unknown-linux-gnu\n");
    Ok(())
});

gentoo_test!(gentoo_eval_amd64_arch, {
    drydock_gentoo()
        .args(["eval", "-p", "gentoo:default/linux/amd64/23.0", "ARCH"])
        .assert()
        .success()
        .stdout("amd64\n");
    Ok(())
});

gentoo_test!(gentoo_eval_use_contains_multilib, {
    let output = drydock_gentoo()
        .args(["eval", "-p", "gentoo:default/linux/amd64/23.0", "USE"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    let tokens: Vec<&str> = stdout.split_whitespace().collect();
    assert!(tokens.contains(&"multilib"), "USE should contain multilib, got: {}", stdout.trim());
    Ok(())
});

gentoo_test!(gentoo_blame_use_multilib, {
    let output = drydock_gentoo()
        .args(["blame", "-p", "gentoo:default/linux/amd64/23.0", "USE:multilib"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    assert!(stdout.contains("SET"));
    assert!(stdout.contains("gentoo:"));
    Ok(())
});

gentoo_test!(gentoo_parents_amd64_has_arch, {
    let output = drydock_gentoo()
        .args(["parents", "gentoo:default/linux/amd64/23.0"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output)?;
    assert!(stdout.contains("gentoo:arch/amd64"));
    Ok(())
});
