# chain-forge-config

Configuration management for Chain Forge projects.

## Overview

Provides TOML-based configuration system with support for multiple profiles and environment-specific settings.

## Features

- **TOML Configuration**: Human-readable config files
- **Multiple Profiles**: Support for dev, test, prod profiles
- **Hierarchical Loading**: Project → Home → Defaults
- **Data Directory Management**: Automatic data directory creation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
chain-forge-config = { path = "../../crates/config" }
```

### Loading Configuration

```rust
use chain_forge_config::Config;

// Load from chain-forge.toml (current dir or home)
let config = Config::load()?;

// Access Solana configuration
if let Some(solana_config) = config.solana {
    let profile = solana_config.default;
    println!("RPC URL: {}", profile.rpc_url);
    println!("Accounts: {}", profile.accounts);
}
```

### Creating a Config File

Create `chain-forge.toml` in your project:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.ci]
accounts = 3
initial_balance = 10.0
port = 8900
```

## Configuration Structure

### Global Config

```rust
pub struct Config {
    pub solana: Option<SolanaConfig>,
    // Future: bitcoin, ethereum, etc.
}
```

### Solana Config

```rust
pub struct SolanaConfig {
    pub default: SolanaProfile,
    pub profiles: HashMap<String, SolanaProfile>,
}

pub struct SolanaProfile {
    pub rpc_url: String,        // Default: "http://localhost:8899"
    pub accounts: u32,          // Default: 10
    pub initial_balance: f64,   // Default: 100.0
    pub port: u16,              // Default: 8899
}
```

## Configuration Priority

1. **Project Config**: `./chain-forge.toml`
2. **Global Config**: `~/.config/chain-forge.toml` or `~/chain-forge.toml`
3. **Built-in Defaults**: Hardcoded defaults

## Data Directory

Chain Forge stores data in `~/.chain-forge/`:

```
~/.chain-forge/
├── solana/
│   └── accounts.json
├── bitcoin/
│   └── wallets.json
└── ...
```

### Using Data Directory

```rust
use chain_forge_config::Config;

// Get data directory path
let data_dir = Config::data_dir();
println!("Data directory: {:?}", data_dir);

// Ensure it exists
let data_dir = Config::ensure_data_dir()?;
```

## Environment Variables

Configuration values can be overridden via environment variables (future feature):

```bash
CHAIN_FORGE_SOLANA_RPC_URL=http://localhost:8900
CHAIN_FORGE_SOLANA_ACCOUNTS=20
```

## Examples

### Multiple Profiles

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0

[solana.devnet]
rpc_url = "https://api.devnet.solana.com"
accounts = 5
initial_balance = 10.0

[solana.mainnet]
rpc_url = "https://api.mainnet-beta.solana.com"
accounts = 1
initial_balance = 0.0
```

### Loading Specific Profile

```rust
let config = Config::load()?;
if let Some(solana_config) = config.solana {
    // Get specific profile
    if let Some(devnet_profile) = solana_config.profiles.get("devnet") {
        println!("Devnet RPC: {}", devnet_profile.rpc_url);
    }
}
```

## Testing

```rust
#[test]
fn test_config_loading() {
    let config = Config::load().unwrap();
    // Config should load with defaults even if no file exists
    assert!(config.solana.is_none() || config.solana.is_some());
}
```

## License

MIT OR Apache-2.0
