// Copyright 2021 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};

/// A blob of all options and configuration specific to drydock.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DrydockConfig {
    #[serde(default)]
    pub src_path: PathBuf,
}

impl DrydockConfig {
    /// Load the configuration file from disk and merge it with options specified from
    /// the command line.
    pub fn load(
        config_path: Option<impl AsRef<Path>>,
        src_path: Option<impl AsRef<Path>>,
    ) -> anyhow::Result<Self> {
        let mut dd_conf = if let Some(ref p) = src_path {
            // src_path on CLI is sufficient; no config file needed.
            DrydockConfig {
                src_path: p.as_ref().to_path_buf(),
            }
        } else {
            let config_path = if let Some(p) = config_path {
                p.as_ref().to_path_buf()
            } else {
                get_default_config_path()?
            };

            config::Config::builder()
                .add_source(config::File::from(config_path))
                .build()
                .with_context(|| {
                    "Unable to find a configuration file. Have you tried running \
                    `drydock config --default`? You can specify a repositories path with --src-path."
                })?
                .try_deserialize()?
        };

        // Resolve tildes in src_path to a concrete directory.
        dd_conf.src_path = dd_conf
            .src_path
            .to_str()
            .map(|s| PathBuf::from(shellexpand::tilde(s).as_ref()))
            .ok_or_else(|| {
                anyhow!(
                    "Unable to shell expand your source path {}",
                    dd_conf.src_path.display()
                )
            })?;

        // Canonicalize paths that are symlinks into the real paths they represent.
        dd_conf.src_path = dd_conf.src_path.canonicalize().with_context(|| {
            format!("Unable to canonicalize path {}", dd_conf.src_path.display())
        })?;

        Ok(dd_conf)
    }

    pub fn save(&self, config_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let config = toml::to_string_pretty(&self)?;
        fs::create_dir_all(config_path.as_ref().parent().unwrap())?;
        let mut file = fs::File::create(config_path)?;
        file.write_all(config.as_bytes())?;
        Ok(())
    }
}

impl Default for DrydockConfig {
    fn default() -> Self {
        DrydockConfig {
            src_path: "/var/db/repos".into(),
        }
    }
}

/// Generate a default configuration file under `$XDG_CONFIG_HOME` or `~/.config/drydock`
///
/// The default source path is `/var/db/repos`. It is an error if the specified path does
/// not exist.
pub fn generate_default(
    config_path: Option<impl AsRef<Path>>,
    src_path: Option<impl AsRef<Path>>,
) -> anyhow::Result<()> {
    // TODO(cjmcdonald): Typed errors would make this function body much simpler.
    let config_path = if let Some(p) = config_path {
        p.as_ref().to_path_buf()
    } else {
        get_default_config_path()?
    };

    let mut config = DrydockConfig::default();

    // If the user specified a src_path argument, use that instead of the default value.
    if let Some(p) = src_path {
        config.src_path = p.as_ref().to_path_buf();
    }

    // Resolve tildes in src_path to a concrete directory.
    config.src_path = config
        .src_path
        .to_str()
        .map(|s| PathBuf::from(shellexpand::tilde(s).as_ref()))
        .ok_or_else(|| {
            anyhow!(
                "Unable to shell expand your source path {}",
                config.src_path.display()
            )
        })?;

    if config.src_path.is_dir() {
        println!("Using {} as your repositories path.", config.src_path.display());
        println!(
            "Edit the config file at {} to change the repositories path.",
            config_path.display()
        );
    } else if let Ok(p) = config.src_path.canonicalize() {
        // A path that is a symlink is fine as long as whatever location it is pointed at is valid.
        // Path canonicalization is done at configuration load time.
        if p.is_dir() {
            println!("Using {} as your repositories path.", config.src_path.display());
            println!(
                "Edit the config file at {} to change the repositories path.",
                config_path.display()
            );
        } else {
            eprintln!("{} is not a valid directory!", p.display());
            bail!(
                "Please re-run this command and specify the path to your repositories directory \
                via the `--src-path` argument."
            )
        }
    } else {
        eprintln!("{} is not a valid directory!", config.src_path.display());
        bail!(
            "Please re-run this command and specify the path to your repositories directory \
            via the `--src-path` argument."
        )
    }
    config.save(&config_path)?;
    println!("Configuration file generated at {}", config_path.display());
    Ok(())
}

/// Helper function to hide away the details of probing the various paths a
/// configuration file might live at.
fn get_default_config_path() -> anyhow::Result<PathBuf> {
    if let Ok(s) = env::var("XDG_CONFIG_HOME") {
        Ok(s.into())
    } else {
        let mut p = PathBuf::from(env::var("HOME")?);
        p.push(".config");
        p.push("drydock");
        p.push("config.toml");
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn test_default_config_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        generate_default(Some(config_path.clone()), Some(temp_dir.path())).unwrap();
    }

    #[test]
    fn test_config_default_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        generate_default(Some(&config_path), Some(temp_dir.path())).unwrap();

        let loaded_config =
            DrydockConfig::load(Some(config_path), Option::<PathBuf>::None).unwrap();

        let default_config = DrydockConfig {
            src_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        assert_eq!(loaded_config, default_config);
    }

    #[test]
    fn test_config_assert_save_and_load_is_same() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config = DrydockConfig {
            src_path: temp_dir.path().to_path_buf(),
        };

        config.save(&config_path)?;

        let loaded_config = DrydockConfig::load(Some(config_path), Option::<PathBuf>::None)?;

        assert_eq!(loaded_config, config);

        Ok(())
    }
}
