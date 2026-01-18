# chain-forge-common

Common traits, types, and utilities for Chain Forge blockchain implementations.

## Overview

This crate provides the foundational abstractions that all blockchain implementations in Chain Forge must implement. It defines the core `ChainProvider` trait and common error types.

## Features

- **ChainProvider Trait**: Abstract interface for blockchain implementations
- **Common Error Types**: Standardized error handling across chains
- **Shared Types**: Network types and common data structures

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
chain-forge-common = { path = "../../crates/common" }
```

### Implementing ChainProvider

```rust
use chain_forge_common::{ChainProvider, Result};

struct MyChainProvider {
    // Your chain-specific state
}

impl ChainProvider for MyChainProvider {
    type Account = MyAccount;
    type Transaction = MyTransaction;
    type Config = MyConfig;

    fn start(&mut self, config: Self::Config) -> Result<()> {
        // Start your blockchain node
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        // Stop your blockchain node
        Ok(())
    }

    fn is_running(&self) -> bool {
        // Check if node is running
        true
    }

    fn get_accounts(&self) -> Result<Vec<Self::Account>> {
        // Return generated accounts
        Ok(vec![])
    }

    fn fund_account(&self, address: &str, amount: f64) -> Result<String> {
        // Fund an account
        Ok("tx_hash".to_string())
    }

    fn get_balance(&self, address: &str) -> Result<f64> {
        // Get account balance
        Ok(0.0)
    }

    fn get_rpc_url(&self) -> String {
        // Return RPC URL
        "http://localhost:8545".to_string()
    }
}
```

## API Reference

### `ChainProvider` Trait

Core trait that all blockchain implementations must implement.

**Associated Types:**
- `Account` - Chain-specific account type
- `Transaction` - Chain-specific transaction type
- `Config` - Chain-specific configuration

**Required Methods:**
- `start(&mut self, config: Self::Config) -> Result<()>` - Start the chain
- `stop(&mut self) -> Result<()>` - Stop the chain
- `is_running(&self) -> bool` - Check if running
- `get_accounts(&self) -> Result<Vec<Self::Account>>` - Get all accounts
- `fund_account(&self, address: &str, amount: f64) -> Result<String>` - Fund account
- `get_balance(&self, address: &str) -> Result<f64>` - Get balance
- `get_rpc_url(&self) -> String` - Get RPC URL

### `ChainError` Enum

Standardized error types:

```rust
pub enum ChainError {
    Config(String),           // Configuration errors
    AccountGeneration(String), // Account generation errors
    Rpc(String),              // RPC errors
    NodeManagement(String),    // Node management errors
    Io(std::io::Error),       // I/O errors
    Serialization(serde_json::Error), // Serialization errors
    NotRunning,               // Chain not running
    AlreadyRunning,           // Chain already running
    Other(String),            // Other errors
}
```

### `Network` Enum

Network types:

```rust
pub enum Network {
    Localnet,  // Local development
    Devnet,    // Developer network
    Testnet,   // Test network
    Mainnet,   // Production network
}
```

## Architecture

This crate is intentionally minimal to keep the abstraction clean. Chain-specific logic should be implemented in chain-specific crates (e.g., `chain-forge-solana-core`).

## License

MIT OR Apache-2.0
