use std::{collections::HashMap, env, net::IpAddr, path::Path};

use config::{Case, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

const DEFAULT_TOML: &str = include_str!("../config/default.toml");

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listener: Listener,
    pub database: Database,
    pub logging: Logging,
    pub oidc: Oidc,
    pub hive: Option<Hive>,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder()
            // Base configuration included in the binary
            .add_source(File::from_str(DEFAULT_TOML, FileFormat::Toml))
            // Defualt location for config file
            .add_source(File::from(Path::new("/etc/dionysus/config.toml")).required(false));

        // If DIONYSUS_CONFIG environment variable is set check path for config takes precedence
        if let Ok(path) = env::var("DIONYSUS_CONFIG") {
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

#[derive(Debug, Deserialize)]
pub struct Logging {
    pub filter: String,
    pub json: bool,
}

#[derive(Debug, Deserialize)]
pub struct Oidc {
    pub external_base_url: String,
    pub providers: HashMap<String, OidcProvider>,
}

#[derive(Debug, Deserialize)]
pub struct Hive {
    pub base_url: String,
    pub token: String,
    pub provider_id: String,
    #[serde(default = "default_hive_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
    #[serde(default = "default_hive_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_hive_cache_ttl_secs() -> u64 {
    60
}

fn default_hive_timeout_secs() -> u64 {
    5
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OidcProvider {
    pub issuer: String,
    pub external_issuer: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
}
