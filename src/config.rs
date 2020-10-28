extern crate config;

use std::{collections::HashMap, env, fs, io::Write, path::Path};

use anyhow::Context;
pub use config::Config;

pub fn get() -> anyhow::Result<Config> {
    let config_path = env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME")? + "/.config")
        + "/drydock/config.toml";
    let mut config = crate::config::Config::new();
    config.merge(config::File::with_name(&config_path)).with_context(|| "Unable to find a configuration file. Have you tried running `drydock config --default`?")?;
    Ok(config)
}

pub fn generate_default() -> anyhow::Result<()> {
    let config_path = env::var("XDG_CONFIG_HOME").unwrap_or(env::var("HOME")? + "/.config")
        + "/drydock/config.toml";
    let mut config = crate::config::Config::new();
    let mut input = String::new();

    print!("Please provide the full path to the `src` directory in your checkout: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;

    // Replace a leading tilde with the user's home directory.
    input = input.replace("~", env::var("HOME")?.as_str());

    input = input.trim().to_owned();

    let input_path: &Path = input.as_ref();
    input_path
        .canonicalize()
        .with_context(|| format!("The path {:?} is an invalid src root.", input_path))?;

    config.set("src_path", input.as_str())?;

    write_config_to_file(config_path.as_ref(), &config)?;
    println!("Configuration file generated at {}", config_path);
    Ok(())
}

fn write_config_to_file(file_path: &Path, config: &Config) -> anyhow::Result<()> {
    let config = config
        .clone()
        .try_into::<HashMap<String, String>>()
        .unwrap();
    let config = toml::to_string(&config)?;
    fs::create_dir_all(file_path.parent().unwrap())?;
    let mut file = fs::File::create(&file_path)?;
    file.write_all(config.as_bytes())?;
    Ok(())
}
