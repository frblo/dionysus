use std::net::IpAddr;

use config::{Case, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listener: Listener,
    pub database: Database,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = config::Config::builder()
            // Start of by merging default configuration
            .add_source(File::with_name("config/default"))
            // Add config file,
            // it is not required
            .add_source(File::with_name("config/dionysus").required(false))
            // Lastly add in env variables
            .add_source(Environment::with_prefix("DIONYSUS")
                .prefix_separator("_")
                .separator("__")
                .convert_case(Case::Snake))
            .build()?;

        builder.try_deserialize()
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
