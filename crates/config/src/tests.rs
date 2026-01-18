use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.solana.is_none());
}

#[test]
fn test_solana_profile_defaults() {
    let profile = SolanaProfile::default();
    assert_eq!(profile.rpc_url, "http://localhost:8899");
    assert_eq!(profile.accounts, 10);
    assert_eq!(profile.initial_balance, 100.0);
    assert_eq!(profile.port, 8899);
}

#[test]
fn test_load_config_from_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chain-forge.toml");

    let toml_content = r#"
[solana.default]
rpc_url = "http://localhost:9000"
accounts = 20
initial_balance = 500.0
port = 9000
    "#;

    fs::write(&config_path, toml_content).unwrap();

    let config = Config::load_from_file(&config_path).unwrap();
    assert!(config.solana.is_some());

    let solana = config.solana.unwrap();
    assert_eq!(solana.default.rpc_url, "http://localhost:9000");
    assert_eq!(solana.default.accounts, 20);
    assert_eq!(solana.default.initial_balance, 500.0);
    assert_eq!(solana.default.port, 9000);
}

#[test]
fn test_load_config_with_multiple_profiles() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chain-forge.toml");

    let toml_content = r#"
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.devnet]
rpc_url = "https://api.devnet.solana.com"
accounts = 5
initial_balance = 10.0
port = 8899
    "#;

    fs::write(&config_path, toml_content).unwrap();

    let config = Config::load_from_file(&config_path).unwrap();
    let solana = config.solana.unwrap();

    // Check default profile
    assert_eq!(solana.default.rpc_url, "http://localhost:8899");

    // Check devnet profile
    assert!(solana.profiles.contains_key("devnet"));
    let devnet = solana.profiles.get("devnet").unwrap();
    assert_eq!(devnet.rpc_url, "https://api.devnet.solana.com");
    assert_eq!(devnet.accounts, 5);
}

#[test]
fn test_data_dir_creation() {
    let data_dir = Config::ensure_data_dir();
    assert!(data_dir.is_ok());
    let path = data_dir.unwrap();
    assert!(path.exists());
    assert!(path.is_dir());
}

#[test]
fn test_config_file_constants() {
    assert_eq!(CONFIG_FILE_NAME, "chain-forge.toml");
    assert_eq!(DATA_DIR_NAME, ".chain-forge");
}

#[test]
fn test_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("chain-forge.toml");

    fs::write(&config_path, "invalid toml {{{").unwrap();

    let result = Config::load_from_file(&config_path);
    assert!(result.is_err());
}

#[test]
fn test_missing_file_returns_default() {
    let config = Config::load().unwrap();
    // Should succeed even if no config file exists
    assert!(config.solana.is_none());
}
