use demostf_client::ApiClient;
use serde::Deserialize;
use std::fs::read_to_string;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config from {path}: {error}")]
    Read { error: std::io::Error, path: String },
    #[error("Failed to parse config from {path}: {error}")]
    Parse {
        error: toml::de::Error,
        path: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub source: StorageConfig,
    pub target: StorageConfig,
    pub migrate: MigrateConfig,
}

impl Config {
    pub fn load(path: String) -> Result<Self, ConfigError> {
        let content = read_to_string(&path).map_err(|error| ConfigError::Read {
            error,
            path: path.clone(),
        })?;
        toml::from_str(&content).map_err(|error| ConfigError::Parse { error, path })
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_base")]
    pub url: String,
    pub key_file: String,
}

fn default_api_base() -> String {
    ApiClient::DEMOS_TF_BASE_URL.into()
}

#[derive(Debug, Deserialize)]
pub struct MigrateConfig {
    pub age: u64,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub root: String,
    pub backend: String,
}
