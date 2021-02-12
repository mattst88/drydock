use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;

pub fn get(args: &clap::ArgMatches) -> anyhow::Result<Config> {
    let config_path = if let Some(s) = args.value_of("config_file") {
        s.to_string()
    } else {
        get_default_config_path()?
    };

    let mut config = config::Config::new();
    config.merge(config::File::with_name(&config_path)).with_context(
        || "Unable to find a configuration file. Have you tried running `drydock config --default`?")?;

    if let Some(src_path) = args.value_of("src_path") {
        config.set("src_path", src_path)?;
    }

    Config::from_dynamic_config(&config)
}

pub fn generate_default(args: &clap::ArgMatches) -> anyhow::Result<()> {
    let home = env::var("HOME").with_context(|| {
        "The HOME variable must be defined in the environment \
        in order to generate a default configuration file."
            .to_string()
    })?;

    let config_path = if let Some(s) = args.value_of("config_file") {
        s.to_string()
    } else {
        get_default_config_path()?
    };

    let mut config = config::Config::new();
    let mut input = String::new();

    let default_checkout_guess = PathBuf::from(home.clone() + "/chromiumos/src");

    // If ~/chromiumos/src exists we assume that it's a source checkout. This is a very common
    // default choice for Chrome OS developers.
    if default_checkout_guess.is_dir() {
        config.set("src_path", default_checkout_guess.to_str().unwrap())?;
        println!("Assuming ~/chromiumos/src is a Chrome OS source checkout.");
        println!(
            "Edit the config file at {} to change the source checkout used.",
            &config_path
        );
    } else {
        print!("Please provide the full path to the `src` directory in your checkout: ");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;

        // Replace a leading tilde with the user's home directory.
        input = input.replace("~", home.as_str());

        input = input.trim().to_owned();

        let input_path: &Path = input.as_ref();
        input_path
            .canonicalize()
            .with_context(|| format!("The path {:?} is an invalid src root.", input_path))?;

        config.set("src_path", input.as_str())?;
    }
    write_config_to_file(config_path.as_ref(), &config)?;
    println!("Configuration file generated at {}", config_path);
    Ok(())
}

fn write_config_to_file(file_path: &Path, config: &config::Config) -> anyhow::Result<()> {
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

pub struct Config {
    pub src_path: PathBuf,
}

impl Config {
    pub fn from_dynamic_config(conf: &config::Config) -> anyhow::Result<Self> {
        let src_path = conf.get_str("src_path")?;
        Ok(Config {
            src_path: src_path.into(),
        })
    }
}

fn get_default_config_path() -> anyhow::Result<String> {
    Ok(match env::var("XDG_CONFIG_HOME") {
        Ok(s) => s,
        Err(_) => env::var("HOME")? + "/.config",
    } + "/drydock/config.toml")
}
