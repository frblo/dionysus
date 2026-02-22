use std::{env, net::IpAddr, path::Path};

use config::{Case, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

const DEFAULT_TOML: &str = include_str!("../config/default.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listener: Listener,
    pub database: Database,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder()
            // Base configuration included in the binary
            .add_source(File::from_str(DEFAULT_TOML, FileFormat::Toml))
            // Defualt location for config file
            .add_source(File::from(Path::new("/etc/dionysus/config.toml")).required(false));

        // If DIONYSUS_CONFIG environment variable is set check path for config takes precedence
        if let Some(path) = env::var("DIONYSUS_CONFIG").ok() {
            builder = builder.add_source(File::from(Path::new(&path)).required(true));
        }

        // Environment variables override file configuration
        builder = builder.add_source(
            Environment::with_prefix("DIONYSUS")
                .prefix_separator("_")
                .separator("__")
                .convert_case(Case::Snake),
        );

        builder.build()?.try_deserialize()
    }
}

#[derive(Debug, Deserialize)]
pub struct Listener {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}
