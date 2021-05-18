// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Integration tests for CLI usage of `drydock`.

mod test_util;

use std::process::Command;

use assert_cmd::prelude::*;
use tempfile;

use test_util::test_data_dir;

#[test]
fn assert_basic_eval_output_looks_right() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    let src_path = test_data_dir(&["test-tree"]);

    // Generate a config file in a tempdir so that we don't accidentally use a user's real
    // config during testing.
    let mut cfg_cmd = Command::cargo_bin("drydock")?;
    cfg_cmd
        .arg("config")
        .arg("--default")
        .arg("--config-file")
        .arg(&config_path)
        .arg("--src-path")
        .arg(&src_path);
    cfg_cmd.assert().success();

    let mut cmd = Command::cargo_bin("drydock")?;
    cmd.arg("--config-file")
        .arg(&config_path)
        .arg("eval")
        .arg("USE")
        .arg("--profile=spam:special_feature/extra_special_feature");

    cmd.assert()
        .success()
        .stdout("my_flag my_other_flag other_feature\n");

    Ok(())
}
