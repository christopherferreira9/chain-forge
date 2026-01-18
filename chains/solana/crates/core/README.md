# chain-forge-solana-core

Core Solana provider implementation for Chain Forge.

## Overview

Implements the `ChainProvider` trait for Solana, providing complete validator lifecycle management, account generation, and blockchain interaction.

## Features

- **Validator Management**: Start/stop `solana-test-validator`
- **Account Generation**: Automatic BIP39/BIP44 account creation
- **Auto-funding**: Pre-fund accounts on startup
- **Process Lifecycle**: Clean process management
- **Configuration**: Flexible configuration options

## Usage

```toml
[dependencies]
chain-forge-solana-core = { path = "../core" }
```

### Basic Usage

```rust
use chain_forge_solana_core::{SolanaProvider, SolanaConfig};
use chain_forge_common::ChainProvider;

fn main() -> Result<()> {
    let config = SolanaConfig {
        rpc_url: "http://localhost:8899".to_string(),
        port: 8899,
        accounts: 10,
        initial_balance: 100.0,
        mnemonic: None,
    };

    let mut provider = SolanaProvider::with_config(config.clone());

    // Start validator
    provider.start(config)?;
    println!("Validator is running at {}", provider.get_rpc_url());

    // Get accounts
    let accounts = provider.get_accounts()?;
    println!("Generated {} accounts", accounts.len());

    // Fund an account
    let signature = provider.fund_account(&accounts[0].public_key, 5.0)?;
    println!("Funded account: {}", signature);

    // Check balance
    let balance = provider.get_balance(&accounts[0].public_key)?;
    println!("Balance: {} SOL", balance);

    // Stop validator
    provider.stop()?;

    Ok(())
}
```

## API Reference

### `SolanaProvider`

Main provider implementing `ChainProvider` trait.

**Constructors:**

```rust
// Create with default configuration
pub fn new() -> Self

// Create with custom configuration
pub fn with_config(config: SolanaConfig) -> Self
```

**ChainProvider Methods:**

```rust
// Start the validator
fn start(&mut self, config: SolanaConfig) -> Result<()>

// Stop the validator
fn stop(&mut self) -> Result<()>

// Check if validator is running
fn is_running(&self) -> bool

// Get all generated accounts
fn get_accounts(&self) -> Result<Vec<SolanaAccount>>

// Fund an account
fn fund_account(&self, address: &str, amount: f64) -> Result<String>

// Get account balance
fn get_balance(&self, address: &str) -> Result<f64>

// Get RPC URL
fn get_rpc_url(&self) -> String
```

**Additional Methods:**

```rust
// Get RPC client reference
pub fn rpc_client(&self) -> Result<&SolanaRpcClient>
```

### `SolanaConfig`

Configuration for Solana provider.

```rust
pub struct SolanaConfig {
    pub rpc_url: String,           // RPC URL
    pub port: u16,                 // RPC port
    pub accounts: u32,             // Number of accounts
    pub initial_balance: f64,      // Initial balance per account
    pub mnemonic: Option<String>,  // Optional mnemonic
}
```

**Defaults:**
- `rpc_url`: "http://localhost:8899"
- `port`: 8899
- `accounts`: 10
- `initial_balance`: 100.0
- `mnemonic`: None (generates new)

## Lifecycle

### Startup Process

1. Load or generate accounts
2. Start `solana-test-validator` process
3. Wait for validator to be ready (up to 60 attempts)
4. Fund all accounts with initial balance
5. Update account balances
6. Save accounts to storage

### Shutdown Process

1. Kill validator process
2. Wait for process to exit
3. Clean up resources

The provider automatically cleans up on `Drop`, so stopping is automatic when going out of scope.

## Examples

### Custom Mnemonic

```rust
let config = SolanaConfig {
    mnemonic: Some("test test test test test test test test test test test junk".to_string()),
    accounts: 5,
    initial_balance: 50.0,
    ..Default::default()
};

let mut provider = SolanaProvider::with_config(config.clone());
provider.start(config)?;

// Will always generate the same 5 accounts
```

### Integration with Config System

```rust
use chain_forge_config::{Config, SolanaProfile};
use chain_forge_solana_core::{SolanaProvider, SolanaConfig};

let config = Config::load()?;
if let Some(solana_config) = config.solana {
    let profile = solana_config.default;
    let provider_config = SolanaConfig::from(profile);

    let mut provider = SolanaProvider::with_config(provider_config.clone());
    provider.start(provider_config)?;
}
```

### Long-running Validator

```rust
use chain_forge_solana_core::{SolanaProvider, SolanaConfig};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let mut provider = SolanaProvider::new();
    provider.start(SolanaConfig::default())?;

    println!("Validator running. Press Ctrl+C to stop.");

    // Wait for Ctrl+C
    signal::ctrl_c().await?;

    println!("Stopping validator...");
    provider.stop()?;

    Ok(())
}
```

### Error Handling

```rust
match provider.start(config) {
    Ok(()) => println!("Validator started successfully"),
    Err(ChainError::AlreadyRunning) => {
        println!("Validator is already running");
    }
    Err(ChainError::NodeManagement(e)) => {
        eprintln!("Failed to start validator: {}", e);
        if e.contains("solana-test-validator not found") {
            eprintln!("Install Solana CLI tools first!");
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Process Management

The provider spawns `solana-test-validator` as a child process:

```bash
solana-test-validator --rpc-port 8899 --quiet --reset
```

### Process Lifecycle

- Process is stored in `Arc<Mutex<Option<Child>>>`
- Automatically killed on `Drop`
- Can be manually stopped with `stop()`
- Stdout/stderr are inherited (visible in terminal)

## Storage

Accounts are saved to `~/.chain-forge/solana/accounts.json` and persist across runs:

- First run: Generates new accounts and saves
- Subsequent runs: Loads existing accounts
- Can be deleted to regenerate

## Prerequisites

**Required:**
- Solana CLI tools installed (`solana-test-validator` must be in PATH)

**Install Solana:**
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

**Verify:**
```bash
solana-test-validator --version
```

## Configuration

### From Code

```rust
let config = SolanaConfig {
    rpc_url: "http://localhost:9000".to_string(),
    port: 9000,
    accounts: 20,
    initial_balance: 500.0,
    mnemonic: None,
};
```

### From Config File

See `chain-forge-config` crate for TOML configuration.

## Testing

```bash
cargo test -p chain-forge-solana-core
```

Integration tests require Solana CLI tools installed.

## Dependencies

- `chain-forge-common` - Common traits and errors
- `chain-forge-config` - Configuration system
- `chain-forge-solana-accounts` - Account generation
- `chain-forge-solana-rpc` - RPC client
- `tokio` - Async runtime
- `solana-sdk` - Solana types

## License

MIT OR Apache-2.0
