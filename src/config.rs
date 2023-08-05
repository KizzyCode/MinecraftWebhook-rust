//! The URL database

use crate::error::Error;
use serde::Deserialize;
use std::{borrow::Cow, collections::BTreeMap, env, ops::Deref};

/// The server config
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// The IP address and port to listen on
    pub address: String,
    /// The connection hart limit; i.e. the amount of threads to spawn at max to process incoming connections
    #[serde(default = "ServerConfig::connection_limit_default")]
    pub connection_limit: usize,
}
impl ServerConfig {
    /// The default value for the connection hard limit
    const fn connection_limit_default() -> usize {
        2048
    }
}

/// The Minecraft server RCON config
#[derive(Debug, Clone, Deserialize)]
pub struct RconConfig {
    /// The IP address and port of the RCON API
    pub address: String,
    /// The RCON password
    pub password: Option<String>,
}

/// The webhook database
#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct WebhookDatabase {
    /// The predefined webhooks
    pub hooks: BTreeMap<String, String>,
}

/// The URL database
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The URL redirects
    pub server: ServerConfig,
    /// The RCON config
    pub rcon: RconConfig,
    /// The webhook database
    pub webhooks: WebhookDatabase,
}
impl Config {
    /// Loads the config from the file
    pub fn load() -> Result<Self, Error> {
        // Get the path from the environment or fallback to a default path
        let path = match env::var("CONFIG_FILE") {
            Ok(path) => Cow::Owned(path),
            Err(_) => Cow::Borrowed("config.toml"),
        };

        // Decode the database
        let data = std::fs::read_to_string(path.deref())?;
        let config: Self = toml::from_str(&data)?;
        Ok(config)
    }
}
