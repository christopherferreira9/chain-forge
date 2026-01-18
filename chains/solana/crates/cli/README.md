# chain-forge-solana-cli

Command-line interface for Chain Forge Solana development.

## Overview

The `cf-solana` CLI tool provides an easy way to start a local Solana validator with pre-funded accounts, manage accounts, and interact with the blockchain.

## Installation

### From Source

```bash
cd chain-forge
cargo install --path chains/solana/crates/cli
```

### Verify Installation

```bash
cf-solana --version
```

## Prerequisites

- Solana CLI tools (for `solana-test-validator`)

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

## Commands

### `cf-solana start`

Start a local Solana test validator with pre-funded accounts.

**Usage:**
```bash
cf-solana start [OPTIONS]
```

**Options:**
- `-a, --accounts <NUM>` - Number of accounts to generate (default: 10)
- `-b, --balance <SOL>` - Initial balance for each account in SOL (default: 100.0)
- `-p, --port <PORT>` - RPC port for the validator (default: 8899)
- `-m, --mnemonic <PHRASE>` - Optional mnemonic phrase for account generation

**Examples:**

```bash
# Start with defaults (10 accounts, 100 SOL each)
cf-solana start

# Start with custom settings
cf-solana start --accounts 20 --balance 500 --port 8900

# Use specific mnemonic for reproducible accounts
cf-solana start --mnemonic "test test test test test test test test test test test junk"

# Minimal setup for CI
cf-solana start --accounts 3 --balance 10
```

**Output:**
```
🔑 Mnemonic: word1 word2 word3 ... word12
   Save this mnemonic to recover your accounts!

🚀 Starting Solana test validator on port 8899...
⏳ Waiting for validator to be ready...
✅ Validator is ready!

💰 Funding 10 accounts with 100 SOL each...
✅ All accounts funded!

🎉 Solana test validator is running!
   RPC URL: http://localhost:8899

💡 Tip: Keep this terminal open to keep the validator running
   Run 'cf-solana accounts' in another terminal to see your accounts
```

**Note:** Press `Ctrl+C` to stop the validator.

### `cf-solana accounts`

List all generated accounts with their balances.

**Usage:**
```bash
cf-solana accounts [OPTIONS]
```

**Options:**
- `-f, --format <FORMAT>` - Output format: `table` or `json` (default: table)

**Examples:**

```bash
# Table format (default)
cf-solana accounts

# JSON format
cf-solana accounts --format json

# Save to file
cf-solana accounts --format json > accounts.json
```

**Table Output:**
```
┌───────┬──────────────────────────────────────────────┬────────────────┐
│ Index │ Public Key                                   │ Balance (SOL)  │
├───────┼──────────────────────────────────────────────┼────────────────┤
│ 0     │ 7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR │ 100.00         │
│ 1     │ 8kL2EwQlX4s5c7Y0Ro9ZnN2xM3yO6dU5bC4tN0iS │ 100.00         │
└───────┴──────────────────────────────────────────────┴────────────────┘
```

**JSON Output:**
```json
[
  {
    "public_key": "7xJ5...",
    "secret_key": [1, 2, 3, ...],
    "mnemonic": "word1 word2 ...",
    "derivation_path": "m/44'/501'/0'/0'",
    "balance": 100.0
  }
]
```

### `cf-solana fund`

Fund an account with SOL via airdrop.

**Usage:**
```bash
cf-solana fund <ADDRESS> <AMOUNT>
```

**Arguments:**
- `ADDRESS` - Public key of the account to fund
- `AMOUNT` - Amount of SOL to send

**Examples:**

```bash
# Fund account with 50 SOL
cf-solana fund 7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR 50

# Fund with decimal amount
cf-solana fund 8kL2EwQlX4s5c7Y0Ro9ZnN2xM3yO6dU5bC4tN0iS 25.5
```

**Output:**
```
💰 Requesting airdrop of 50 SOL to 7xJ5...
✅ Airdrop successful!
   Signature: 5Kc7...
   New balance: 150 SOL
```

### `cf-solana config`

Show current configuration settings.

**Usage:**
```bash
cf-solana config
```

**Output:**
```
Chain Forge Configuration
========================

Solana:
  Default Profile:
    RPC URL: http://localhost:8899
    Accounts: 10
    Initial Balance: 100 SOL
    Port: 8899

  Other Profiles:
    devnet:
      RPC URL: https://api.devnet.solana.com

Data Directory: /Users/username/.chain-forge
```

### `cf-solana stop`

Stop the running validator (info message).

**Usage:**
```bash
cf-solana stop
```

**Note:** Currently, use `Ctrl+C` to stop the validator. This command shows info for future daemon mode support.

## Configuration File

Create `chain-forge.toml` in your project directory:

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

## Typical Workflows

### Development Workflow

```bash
# Terminal 1: Start validator
cf-solana start

# Terminal 2: Check accounts
cf-solana accounts

# Terminal 2: Fund account for testing
cf-solana fund <address> 10

# Terminal 2: Build and test your app
cargo build
cargo test
```

### CI/CD Workflow

```bash
# In your CI script
cf-solana start --accounts 3 --balance 10 &
sleep 5  # Wait for startup

# Run tests
npm test

# Validator automatically stops when script exits
```

### Team Workflow (Shared Mnemonic)

```bash
# Team member 1: Share mnemonic with team
cf-solana start
# Save mnemonic: "word1 word2 ..."

# Team member 2: Use same accounts
cf-solana start --mnemonic "word1 word2 ..."
```

## Data Storage

Chain Forge stores data in `~/.chain-forge/`:

```
~/.chain-forge/
└── solana/
    └── accounts.json  # Generated accounts
```

### Security Warning

⚠️ The `accounts.json` file contains private keys. **Never commit this file** or share it publicly.

Add to `.gitignore`:
```
.chain-forge/
```

## Troubleshooting

### Validator Won't Start

**Error:** `solana-test-validator not found`

**Solution:** Install Solana CLI tools:
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### Port Already in Use

**Error:** Port 8899 is already in use

**Solutions:**
1. Stop existing validator:
   ```bash
   pkill solana-test-validator
   ```

2. Use different port:
   ```bash
   cf-solana start --port 8900
   ```

### No Accounts Found

**Error:** `cf-solana accounts` shows no accounts

**Solution:** Start validator first:
```bash
cf-solana start
```

### Airdrop Failed

**Error:** Airdrop request failed

**Possible causes:**
1. Validator not running - run `cf-solana start`
2. Invalid address - check the address format
3. Rate limit - wait a moment and try again

## Environment Variables

(Future feature - not yet implemented)

```bash
export CHAIN_FORGE_SOLANA_RPC_URL=http://localhost:8900
export CHAIN_FORGE_SOLANA_ACCOUNTS=20
cf-solana start
```

## Shell Completion

Generate shell completion scripts:

```bash
# Bash
cf-solana completion bash > ~/.local/share/bash-completion/completions/cf-solana

# Zsh
cf-solana completion zsh > ~/.zfunc/_cf-solana

# Fish
cf-solana completion fish > ~/.config/fish/completions/cf-solana.fish
```

(Note: Completion generation not yet implemented - planned feature)

## Exit Codes

- `0` - Success
- `1` - Error occurred

## Getting Help

```bash
# Show help
cf-solana --help

# Show command help
cf-solana start --help
cf-solana accounts --help
cf-solana fund --help
```

## Examples

See `examples/` directory for complete examples:
- `examples/typescript-basic/` - TypeScript usage

## License

MIT OR Apache-2.0
