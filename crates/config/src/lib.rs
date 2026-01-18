use chain_forge_common::{ChainError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub const CONFIG_FILE_NAME: &str = "chain-forge.toml";
pub const DATA_DIR_NAME: &str = ".chain-forge";

/// Global configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub solana: Option<SolanaConfig>,
    // Future chains
    // pub bitcoin: Option<BitcoinConfig>,
    // pub ethereum: Option<EthereumConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    #[serde(default)]
    pub default: SolanaProfile,

    #[serde(flatten)]
    pub profiles: std::collections::HashMap<String, SolanaProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaProfile {
    #[serde(default = "default_rpc_url")]
    pub rpc_url: String,

    #[serde(default = "default_accounts")]
    pub accounts: u32,

    #[serde(default = "default_initial_balance")]
    pub initial_balance: f64,

    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for SolanaProfile {
    fn default() -> Self {
        Self {
            rpc_url: default_rpc_url(),
            accounts: default_accounts(),
            initial_balance: default_initial_balance(),
            port: default_port(),
        }
    }
}

fn default_rpc_url() -> String {
    "http://localhost:8899".to_string()
}

fn default_accounts() -> u32 {
    10
}

fn default_initial_balance() -> f64 {
    100.0
}

fn default_port() -> u16 {
    8899
}

impl Config {
    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ChainError::Config(format!(
                "Failed to read config file {:?}: {}",
                path.as_ref(),
                e
            ))
        })?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| ChainError::TomlParsing(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Load configuration from the current directory or user's home directory
    pub fn load() -> Result<Self> {
        // Try current directory first
        let current_dir_config = std::env::current_dir()
            .ok()
            .map(|p| p.join(CONFIG_FILE_NAME));

        if let Some(config_path) = current_dir_config {
            if config_path.exists() {
                return Self::load_from_file(config_path);
            }
        }

        // Try home directory
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(CONFIG_FILE_NAME);
            if config_path.exists() {
                return Self::load_from_file(config_path);
            }
        }

        // Return default configuration
        Ok(Self::default())
    }

    /// Get the data directory path
    pub fn data_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(DATA_DIR_NAME)
    }

    /// Ensure data directory exists
    pub fn ensure_data_dir() -> Result<PathBuf> {
        let dir = Self::data_dir();
        std::fs::create_dir_all(&dir).map_err(|e| {
            ChainError::Config(format!("Failed to create data directory {:?}: {}", dir, e))
        })?;
        Ok(dir)
    }
}
