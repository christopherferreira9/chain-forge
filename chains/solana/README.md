# Chain Forge - Solana

Local Solana development made easy. Start a test validator with pre-funded accounts in seconds.

## Features

- 🔑 **BIP39/BIP44 Account Generation** - Deterministic keypair derivation with mnemonic recovery
- 💰 **Pre-funded Accounts** - Automatically airdrop SOL to generated accounts
- 🚀 **Quick Setup** - Single command to start a local validator
- 📦 **TypeScript Support** - NPM package for programmatic access
- ⚙️ **Configurable** - Customize accounts, balances, and ports via CLI or config file

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/christopherferreira9/chain-forge
cd chain-forge

# Build and install the CLI
cargo install --path chains/solana/crates/cli

# Verify installation
cf-solana --version
```

### Prerequisites

- Rust 1.75 or later
- Solana CLI tools (for `solana-test-validator`)

**Install Solana CLI:**

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

## Quick Start

### Start Validator

```bash
# Start with default settings (10 accounts, 100 SOL each)
cf-solana start

# Custom configuration
cf-solana start --accounts 20 --balance 500 --port 8899
```

### List Accounts

```bash
# Table format
cf-solana accounts

# JSON format
cf-solana accounts --format json
```

### Fund an Account

```bash
cf-solana fund <ADDRESS> 50.0
```

### View Configuration

```bash
cf-solana config
```

## CLI Commands

### `cf-solana start`

Start a local Solana test validator with pre-funded accounts.

**Options:**
- `-a, --accounts <NUM>` - Number of accounts to generate (default: 10)
- `-b, --balance <SOL>` - Initial balance for each account (default: 100.0)
- `-p, --port <PORT>` - RPC port (default: 8899)
- `-m, --mnemonic <PHRASE>` - Use specific mnemonic for account generation

**Example:**
```bash
cf-solana start --accounts 5 --balance 1000
```

### `cf-solana accounts`

List all generated accounts with their balances.

**Options:**
- `-f, --format <FORMAT>` - Output format: `table` or `json` (default: table)

**Example:**
```bash
cf-solana accounts --format json > accounts.json
```

### `cf-solana fund`

Fund an account with SOL via airdrop.

**Arguments:**
- `ADDRESS` - Public key of the account to fund
- `AMOUNT` - Amount of SOL to send

**Example:**
```bash
cf-solana fund 7xJ5... 25.5
```

### `cf-solana config`

Display current configuration settings.

**Example:**
```bash
cf-solana config
```

## Configuration File

Create a `chain-forge.toml` in your project directory:

```toml
[solana.default]
rpc_url = "http://localhost:8899"
accounts = 10
initial_balance = 100.0
port = 8899

[solana.devnet]
rpc_url = "https://api.devnet.solana.com"
```

## Account Generation

Chain Forge uses BIP39 mnemonics and BIP44 derivation paths for account generation:

- **Mnemonic**: 12-word phrase (generated automatically or provided)
- **Derivation Path**: `m/44'/501'/{index}'/0'` (Solana standard)
- **Storage**: Accounts saved to `~/.chain-forge/solana/accounts.json`

### Using a Custom Mnemonic

```bash
cf-solana start --mnemonic "your twelve word mnemonic phrase goes here"
```

This allows you to:
- Recover the same accounts across sessions
- Share accounts with team members
- Integrate with existing wallet setups

## Integration Examples

### Rust

```rust
use chain_forge_solana_core::{SolanaProvider, SolanaConfig};
use chain_forge_common::ChainProvider;

let config = SolanaConfig {
    accounts: 5,
    initial_balance: 100.0,
    port: 8899,
    ..Default::default()
};

let mut provider = SolanaProvider::with_config(config);
provider.start(config)?;

let accounts = provider.get_accounts()?;
println!("First account: {}", accounts[0].public_key);
```

### TypeScript

See the [@chain-forge/solana NPM package](../../npm/@chain-forge/solana) for TypeScript usage.

## Architecture

```
chains/solana/
├── crates/
│   ├── cli/         # Command-line interface binary
│   ├── core/        # Core logic and ChainProvider implementation
│   ├── accounts/    # BIP39/BIP44 account generation
│   └── rpc/         # Solana RPC client wrapper
```

### Key Components

- **CLI** - User-facing command-line tool
- **Core** - Validator lifecycle management and ChainProvider trait implementation
- **Accounts** - Keypair generation using BIP39 mnemonics and BIP44 derivation
- **RPC** - Wrapper around `solana-client` for balance queries and airdrops

## Testing

```bash
# Run all tests
cargo test --workspace

# Run Solana-specific tests
cargo test -p chain-forge-solana-core
cargo test -p chain-forge-solana-accounts
```

## Troubleshooting

### Validator Won't Start

**Issue**: `solana-test-validator not found`

**Solution**: Install Solana CLI tools:
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### Port Already in Use

**Issue**: Port 8899 is already in use

**Solution**: Specify a different port:
```bash
cf-solana start --port 8900
```

### Accounts Not Found

**Issue**: `cf-solana accounts` shows no accounts

**Solution**: Start the validator first to generate accounts:
```bash
cf-solana start
```

## Data Storage

Chain Forge stores data in `~/.chain-forge/`:

```
~/.chain-forge/
└── solana/
    └── accounts.json  # Generated accounts with keys
```

**Security Note**: The `accounts.json` file contains private keys. Never commit this file or share it publicly.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0
