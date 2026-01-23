<p align="center">
  <img src="docs/assets/logo.png" alt="Chain Forge Logo" style="max-width:150px; width:100%;">
</p>



# Chain Forge

A Foundry-inspired multi-chain development tool suite for local blockchain development and testing.

[![CI](https://github.com/christopherferreira9/chain-forge/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/christopherferreira9/chain-forge/actions/workflows/ci.yml)
[![Security Audit](https://github.com/christopherferreira9/chain-forge/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/christopherferreira9/chain-forge/actions/workflows/security.yml)

## Overview

Chain Forge provides a unified interface for working with multiple blockchain networks locally. Currently supports Solana, with Bitcoin and Ethereum support planned for future releases.

## Features

- 🚀 **Multi-chain support** - Unified interface for different blockchain networks
- 🔑 **Account management** - BIP39/BIP44 key derivation with pre-funded accounts
- 🛠️ **CLI tools** - Easy-to-use command-line interface for each chain
- 📦 **TypeScript packages** - Programmatic access via NPM packages
- ⚙️ **Configuration system** - Flexible TOML-based configuration

## Installation

### CLI Tools

```bash
cargo install chain-forge-solana-cli
```

### TypeScript Package

```bash
yarn install @chain-forge/solana
# or
yarn add @chain-forge/solana
```

## Quick Start

### Solana CLI

```bash
# Start local Solana validator with 10 pre-funded accounts
cf-solana start --accounts 10 --balance 100

# List all accounts
cf-solana accounts

# Fund an account
cf-solana fund <address> 50.0

# Show configuration
cf-solana config
```

### TypeScript

```typescript
import { SolanaClient } from '@chain-forge/solana';

const client = new SolanaClient({
  accounts: 10,
  initialBalance: 100
});

await client.start();

const accounts = await client.getAccounts();
console.log(`First account: ${accounts[0].publicKey}`);

await client.fundAccount(accounts[0].publicKey, 5);
await client.stop();
```

## Project Structure

```
chain-forge/
├── chains/           # Chain-specific implementations
│   ├── solana/      # Solana support
│   ├── bitcoin/     # Future: Bitcoin support
│   └── ethereum/    # Future: Ethereum support
├── crates/          # Common utilities
│   ├── common/      # Shared traits and utilities
│   ├── config/      # Configuration system
│   └── cli-utils/   # CLI helpers
└── npm/             # TypeScript packages
```

## Configuration

Create a `chain-forge.toml` in your project:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.devnet]
rpc_url = "https://api.devnet.solana.com"
```

## Documentation

- [Getting Started Guide](docs/guide/getting-started.md)
- [Contributing Guide](docs/contributing/development.md)
- [Testing Guide](docs/contributing/testing.md)

For development documentation, see the [.claude](.claude) directory.

## Development

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build Solana CLI
cargo build -p chain-forge-solana-cli
```

## License

MIT OR Apache-2.0
