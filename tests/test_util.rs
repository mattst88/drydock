// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Common utility functions useful in integration test modules.

use std::path::{Path, PathBuf};

/// Returns the full path to the specified subdir of the `resources/test` directory of this project.
pub(crate) fn test_data_dir<I, P>(subdir_components: I) -> PathBuf
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let basedir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut dir: PathBuf = [basedir.as_str(), "resources", "test"].iter().collect();
    dir.extend(subdir_components.into_iter());
    dir
}
